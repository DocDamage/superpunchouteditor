//! WAV file reading/writing and audio resampling.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::brr::{BrrDecoder, BrrEncodeOptions, BrrEncoder};

use super::constants::{
    SPC_SAMPLE_RATE, WAV_DATA_MAGIC, WAV_FMT_MAGIC, WAV_RIFF_MAGIC, WAV_WAVE_MAGIC,
};

/// WAV file error types.
#[derive(Debug, Clone)]
pub enum WavError {
    /// File not found
    NotFound(String),
    /// Invalid format
    InvalidFormat(String),
    /// IO error
    IoError(String),
    /// Unsupported format
    UnsupportedFormat(String),
}

impl std::fmt::Display for WavError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WavError::NotFound(path) => write!(f, "WAV file not found: {}", path),
            WavError::InvalidFormat(msg) => write!(f, "Invalid WAV format: {}", msg),
            WavError::IoError(msg) => write!(f, "IO error: {}", msg),
            WavError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

impl std::error::Error for WavError {}

impl From<std::io::Error> for WavError {
    fn from(err: std::io::Error) -> Self {
        WavError::IoError(err.to_string())
    }
}

/// Reads a WAV file and returns PCM samples.
///
/// Supports 16-bit PCM mono and stereo WAV files.
/// Returns mono samples (stereo is mixed down to mono).
///
/// # Arguments
/// - `path`: Path to the WAV file
///
/// # Returns
/// A tuple of (pcm_samples, sample_rate)
///
/// # Example
/// ```
/// use asset_core::audio::read_wav_file;
///
/// // let (pcm, rate) = read_wav_file("sound.wav").unwrap();
/// ```
pub fn read_wav_file<P: AsRef<Path>>(path: P) -> Result<(Vec<i16>, u32), WavError> {
    let path = path.as_ref();
    let mut file =
        File::open(path).map_err(|e| WavError::NotFound(format!("{}: {}", path.display(), e)))?;

    // Read RIFF header
    let mut riff_header = [0u8; 12];
    file.read_exact(&mut riff_header)?;

    if &riff_header[0..4] != WAV_RIFF_MAGIC {
        return Err(WavError::InvalidFormat("Not a RIFF file".to_string()));
    }

    if &riff_header[8..12] != WAV_WAVE_MAGIC {
        return Err(WavError::InvalidFormat("Not a WAVE file".to_string()));
    }

    // Parse chunks
    let mut fmt_chunk = None;
    let mut data_chunk = None;

    loop {
        let mut chunk_header = [0u8; 8];
        match file.read_exact(&mut chunk_header) {
            Ok(_) => {}
            Err(_) => break, // End of file
        }

        let chunk_id = &chunk_header[0..4];
        let chunk_size = u32::from_le_bytes([
            chunk_header[4],
            chunk_header[5],
            chunk_header[6],
            chunk_header[7],
        ]) as usize;

        if chunk_id == WAV_FMT_MAGIC {
            let mut fmt_data = vec![0u8; chunk_size];
            file.read_exact(&mut fmt_data)?;
            fmt_chunk = Some(fmt_data);
        } else if chunk_id == WAV_DATA_MAGIC {
            let mut data = vec![0u8; chunk_size];
            file.read_exact(&mut data)?;
            data_chunk = Some(data);
        } else {
            // Skip unknown chunk
            let mut skip = vec![0u8; chunk_size];
            file.read_exact(&mut skip)?;
        }

        // Align to word boundary
        if chunk_size % 2 != 0 {
            let mut pad = [0u8; 1];
            let _ = file.read_exact(&mut pad);
        }
    }

    let fmt = fmt_chunk.ok_or_else(|| WavError::InvalidFormat("Missing fmt chunk".to_string()))?;
    let data =
        data_chunk.ok_or_else(|| WavError::InvalidFormat("Missing data chunk".to_string()))?;

    // Parse fmt chunk
    let audio_format = u16::from_le_bytes([fmt[0], fmt[1]]);
    let num_channels = u16::from_le_bytes([fmt[2], fmt[3]]);
    let sample_rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
    let bits_per_sample = u16::from_le_bytes([fmt[14], fmt[15]]);

    if audio_format != 1 {
        return Err(WavError::UnsupportedFormat(format!(
            "Audio format {} not supported (only PCM)",
            audio_format
        )));
    }

    if bits_per_sample != 16 {
        return Err(WavError::UnsupportedFormat(format!(
            "Bits per sample {} not supported (only 16-bit)",
            bits_per_sample
        )));
    }

    // Convert bytes to samples
    let sample_count = data.len() / 2;
    let mut samples = Vec::with_capacity(sample_count);

    for i in 0..sample_count {
        let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        samples.push(sample);
    }

    // Mix stereo to mono if needed
    if num_channels == 2 {
        let mono_samples: Vec<i16> = samples
            .chunks(2)
            .map(|stereo| {
                let left = stereo[0] as i32;
                let right = stereo[1] as i32;
                ((left + right) / 2) as i16
            })
            .collect();
        Ok((mono_samples, sample_rate))
    } else {
        Ok((samples, sample_rate))
    }
}

/// Writes PCM samples to a WAV file.
///
/// Writes 16-bit PCM mono WAV.
///
/// # Arguments
/// - `path`: Output file path
/// - `pcm`: PCM samples
/// - `sample_rate`: Sample rate in Hz
///
/// # Example
/// ```
/// use asset_core::audio::write_wav_file;
///
/// // let pcm = vec![0i16; 1000];
/// // write_wav_file("output.wav", &pcm, 32040).unwrap();
/// ```
pub fn write_wav_file<P: AsRef<Path>>(
    path: P,
    pcm: &[i16],
    sample_rate: u32,
) -> Result<(), WavError> {
    let path = path.as_ref();
    let mut file = File::create(path)?;

    let data_size = pcm.len() * 2;
    let file_size = 36 + data_size;

    // RIFF header
    file.write_all(WAV_RIFF_MAGIC)?;
    file.write_all(&(file_size as u32).to_le_bytes())?;
    file.write_all(WAV_WAVE_MAGIC)?;

    // fmt chunk
    file.write_all(WAV_FMT_MAGIC)?;
    file.write_all(&16u32.to_le_bytes())?; // Chunk size
    file.write_all(&1u16.to_le_bytes())?; // Audio format (PCM)
    file.write_all(&1u16.to_le_bytes())?; // Number of channels (mono)
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * 2).to_le_bytes())?; // Byte rate
    file.write_all(&2u16.to_le_bytes())?; // Block align
    file.write_all(&16u16.to_le_bytes())?; // Bits per sample

    // data chunk
    file.write_all(WAV_DATA_MAGIC)?;
    file.write_all(&(data_size as u32).to_le_bytes())?;

    // Write samples
    for sample in pcm {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

/// Writes stereo PCM samples to a WAV file.
///
/// Writes 16-bit PCM stereo WAV.
///
/// # Arguments
/// - `path`: Output file path
/// - `left`: Left channel PCM samples
/// - `right`: Right channel PCM samples
/// - `sample_rate`: Sample rate in Hz
pub fn write_wav_stereo<P: AsRef<Path>>(
    path: P,
    left: &[i16],
    right: &[i16],
    sample_rate: u32,
) -> Result<(), WavError> {
    let path = path.as_ref();
    let mut file = File::create(path)?;

    let sample_count = left.len().min(right.len());
    let data_size = sample_count * 4; // 2 bytes per sample, 2 channels
    let file_size = 36 + data_size;

    // RIFF header
    file.write_all(WAV_RIFF_MAGIC)?;
    file.write_all(&(file_size as u32).to_le_bytes())?;
    file.write_all(WAV_WAVE_MAGIC)?;

    // fmt chunk
    file.write_all(WAV_FMT_MAGIC)?;
    file.write_all(&16u32.to_le_bytes())?; // Chunk size
    file.write_all(&1u16.to_le_bytes())?; // Audio format (PCM)
    file.write_all(&2u16.to_le_bytes())?; // Number of channels (stereo)
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * 4).to_le_bytes())?; // Byte rate
    file.write_all(&4u16.to_le_bytes())?; // Block align
    file.write_all(&16u16.to_le_bytes())?; // Bits per sample

    // data chunk
    file.write_all(WAV_DATA_MAGIC)?;
    file.write_all(&(data_size as u32).to_le_bytes())?;

    // Write interleaved samples
    for i in 0..sample_count {
        file.write_all(&left[i].to_le_bytes())?;
        file.write_all(&right[i].to_le_bytes())?;
    }

    Ok(())
}

/// Resamples audio using linear interpolation.
///
/// # Arguments
/// - `input`: Input PCM samples
/// - `input_rate`: Input sample rate
/// - `output_rate`: Output sample rate
///
/// # Returns
/// Resampled PCM samples
pub fn resample_linear(input: &[i16], input_rate: u32, output_rate: u32) -> Vec<i16> {
    if input_rate == output_rate || input.is_empty() {
        return input.to_vec();
    }

    let ratio = input_rate as f64 / output_rate as f64;
    let output_len = ((input.len() as f64) / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let input_pos = i as f64 * ratio;
        let input_idx = input_pos as usize;
        let frac = input_pos - input_idx as f64;

        let s1 = input[input_idx.min(input.len() - 1)] as f64;
        let s2 = input[(input_idx + 1).min(input.len() - 1)] as f64;

        let sample = s1 + (s2 - s1) * frac;
        output.push(sample.clamp(-32768.0, 32767.0) as i16);
    }

    output
}

/// Resamples audio to target sample rate.
///
/// # Arguments
/// - `pcm`: Input PCM samples
/// - `source_rate`: Source sample rate in Hz
/// - `target_rate`: Target sample rate in Hz
///
/// # Returns
/// Resampled PCM samples at target rate
pub fn resample_to_rate(pcm: &[i16], source_rate: u32, target_rate: u32) -> Vec<i16> {
    resample_linear(pcm, source_rate, target_rate)
}

/// Exports a BRR sample to WAV format.
///
/// # Arguments
/// - `brr_data`: BRR encoded data
/// - `output_path`: Output WAV file path
/// - `target_rate`: Target sample rate (will resample if different from SPC rate)
///
/// # Returns
/// `Ok(())` on success, `Err(WavError)` on failure
pub fn export_brr_to_wav<P: AsRef<Path>>(
    brr_data: &[u8],
    output_path: P,
    target_rate: u32,
) -> Result<(), WavError> {
    let decoder = BrrDecoder::new();
    let pcm = decoder.decode(brr_data);

    // Resample if needed
    let final_pcm = if target_rate != SPC_SAMPLE_RATE {
        resample_to_rate(&pcm, SPC_SAMPLE_RATE, target_rate)
    } else {
        pcm
    };

    write_wav_file(output_path, &final_pcm, target_rate)
}

/// Imports a WAV file and encodes to BRR.
///
/// # Arguments
/// - `wav_path`: Path to WAV file
/// - `options`: BRR encoding options
///
/// # Returns
/// Encoded BRR data
pub fn import_wav_to_brr<P: AsRef<Path>>(
    wav_path: P,
    options: BrrEncodeOptions,
) -> Result<Vec<u8>, WavError> {
    let (pcm, source_rate) = read_wav_file(wav_path)?;

    // Resample if needed
    let final_pcm = if source_rate != options.sample_rate {
        resample_to_rate(&pcm, source_rate, options.sample_rate)
    } else {
        pcm
    };

    // Clamp to valid SNES range
    let clamped_pcm: Vec<i16> = final_pcm.iter().map(|&s| s.clamp(-16384, 16383)).collect();

    let encoder = BrrEncoder::new();
    Ok(encoder.encode(&clamped_pcm, options))
}
