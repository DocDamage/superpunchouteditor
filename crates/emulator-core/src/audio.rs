//! # Audio buffer handling
//!
//! Manages audio sample buffering, resampling, and playback configuration
//! for the emulator core.

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;

/// Audio configuration
#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// Number of audio channels (typically 2 for stereo)
    pub channels: usize,
    /// Buffer size in samples
    pub buffer_size: usize,
    /// Enable audio output
    pub enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: crate::SNES_SAMPLE_RATE,
            channels: crate::SNES_AUDIO_CHANNELS,
            buffer_size: 2048,
            enabled: true,
        }
    }
}

impl AudioConfig {
    /// Create a new audio configuration
    pub fn new(sample_rate: f64, channels: usize) -> Self {
        Self {
            sample_rate,
            channels,
            ..Default::default()
        }
    }

    /// Set the sample rate
    pub fn with_sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Set the number of channels
    pub fn with_channels(mut self, channels: usize) -> Self {
        self.channels = channels;
        self
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable audio
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Audio sample batch data
#[derive(Debug, Clone)]
pub struct AudioBatch {
    /// Interleaved audio samples (stereo: LRLRLR...)
    pub samples: Vec<i16>,
    /// Sample rate
    pub sample_rate: f64,
    /// Number of channels
    pub channels: usize,
}

impl AudioBatch {
    /// Create a new audio batch
    pub fn new(samples: Vec<i16>, sample_rate: f64, channels: usize) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    /// Get the number of frames (samples per channel)
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels
    }

    /// Get duration in seconds
    pub fn duration(&self) -> f64 {
        self.frame_count() as f64 / self.sample_rate
    }

    /// Convert to f32 samples (normalized to -1.0..1.0)
    pub fn to_f32(&self) -> Vec<f32> {
        self.samples.iter().map(|&s| s as f32 / 32768.0).collect()
    }

    /// Resample to a different sample rate using simple linear interpolation
    pub fn resample(&self, target_rate: f64) -> Self {
        if (self.sample_rate - target_rate).abs() < 0.1 {
            return self.clone();
        }

        let ratio = target_rate / self.sample_rate;
        let new_frame_count = (self.frame_count() as f64 * ratio) as usize;
        let mut new_samples = Vec::with_capacity(new_frame_count * self.channels);

        for i in 0..new_frame_count {
            let src_pos = i as f64 / ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;

            for ch in 0..self.channels {
                let idx1 = (src_idx * self.channels + ch).min(self.samples.len() - 1);
                let idx2 = ((src_idx + 1) * self.channels + ch).min(self.samples.len() - 1);

                let s1 = self.samples[idx1] as f64;
                let s2 = self.samples[idx2] as f64;
                let interpolated = (s1 + (s2 - s1) * frac) as i16;

                new_samples.push(interpolated);
            }
        }

        Self::new(new_samples, target_rate, self.channels)
    }
}

/// Thread-safe audio buffer manager
pub struct AudioBuffer {
    /// Internal sample buffer
    buffer: Mutex<Vec<i16>>,
    /// Audio configuration
    config: parking_lot::RwLock<AudioConfig>,
    /// Channel for sending audio to playback thread
    sender: Mutex<Option<Sender<AudioBatch>>>,
    /// Total samples processed
    total_samples: std::sync::atomic::AtomicU64,
}

impl AudioBuffer {
    /// Create a new audio buffer
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(4096)),
            config: parking_lot::RwLock::new(AudioConfig::default()),
            sender: Mutex::new(None),
            total_samples: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create with specific configuration
    pub fn with_config(config: AudioConfig) -> Self {
        let buffer = Self::new();
        *buffer.config.write() = config;
        buffer
    }

    /// Set up audio output channel
    pub fn set_output_channel(&self, sender: Sender<AudioBatch>) {
        *self.sender.lock() = Some(sender);
    }

    /// Submit a single stereo sample (legacy libretro callback)
    pub fn submit_sample(&self, left: i16, right: i16) {
        if !self.config.read().enabled {
            return;
        }

        let mut buffer = self.buffer.lock();
        buffer.push(left);
        buffer.push(right);

        // Flush when buffer is full
        let config = *self.config.read();
        if buffer.len() >= config.buffer_size * config.channels {
            self.flush_buffer(&mut buffer);
        }
    }

    /// Submit a batch of samples (modern libretro callback)
    pub fn submit_batch(&self, data: *const i16, frames: usize) -> usize {
        if !self.config.read().enabled || data.is_null() || frames == 0 {
            return 0;
        }

        // # Safety
        // The pointer must be valid and point to at least `frames * 2` bytes (i16 samples) of initialized memory.
        // The caller must ensure the data remains valid for the lifetime of this function call.
        // This is called from the libretro audio callback which provides valid sample data.
        let samples = unsafe { std::slice::from_raw_parts(data, frames * 2) };
        let mut buffer = self.buffer.lock();
        buffer.extend_from_slice(samples);

        let config = *self.config.read();
        if buffer.len() >= config.buffer_size * config.channels {
            self.flush_buffer(&mut buffer);
        }

        frames
    }

    /// Flush the internal buffer to the output channel
    fn flush_buffer(&self, buffer: &mut Vec<i16>) {
        if buffer.is_empty() {
            return;
        }

        let config = *self.config.read();
        let batch = AudioBatch::new(buffer.clone(), config.sample_rate, config.channels);

        if let Some(ref sender) = *self.sender.lock() {
            // Non-blocking send, drop samples if channel is full
            let _ = sender.try_send(batch);
        }

        self.total_samples
            .fetch_add(buffer.len() as u64, std::sync::atomic::Ordering::SeqCst);

        buffer.clear();
    }

    /// Get and clear the current buffer contents
    pub fn drain_samples(&self) -> Vec<i16> {
        std::mem::take(&mut *self.buffer.lock())
    }

    /// Get current buffer size in samples
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Set audio configuration
    pub fn set_config(&self, config: AudioConfig) {
        *self.config.write() = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> AudioConfig {
        *self.config.read()
    }

    /// Get total samples processed
    pub fn total_samples(&self) -> u64 {
        self.total_samples.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Enable audio output
    pub fn enable(&self) {
        self.config.write().enabled = true;
    }

    /// Disable audio output
    pub fn disable(&self) {
        self.config.write().enabled = false;
        self.buffer.lock().clear();
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio playback handle for CPAL integration (when audio feature is enabled)
#[cfg(feature = "audio")]
pub struct AudioPlayback {
    /// Audio output stream
    _stream: cpal::Stream,
    /// Sample receiver
    _receiver: Receiver<AudioBatch>,
    /// Resampling buffer
    _resample_buffer: Mutex<Vec<f32>>,
}

#[cfg(feature = "audio")]
use cpal::traits::StreamTrait;

#[cfg(feature = "audio")]
impl AudioPlayback {
    /// Create a new audio playback instance
    pub fn new(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> crate::Result<(Self, Sender<AudioBatch>)> {
        use cpal::traits::DeviceTrait;

        let (sender, receiver) = bounded(16);
        let sample_rate = config.sample_rate.0 as f64;
        let _channels = config.channels as usize;

        let receiver_clone: Receiver<AudioBatch> = receiver.clone();
        let mut local_buffer: Vec<f32> = Vec::with_capacity(1024);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Fill output buffer with samples
                    for sample in data.iter_mut() {
                        if local_buffer.is_empty() {
                            // Try to get more samples from the channel
                            if let Ok(batch) = receiver_clone.try_recv() {
                                // Resample if necessary
                                let resampled = if (batch.sample_rate - sample_rate).abs() > 1.0 {
                                    batch.resample(sample_rate)
                                } else {
                                    batch
                                };

                                // Convert to f32 and add to local buffer
                                local_buffer.extend(resampled.to_f32());
                            }
                        }

                        *sample = local_buffer.pop().unwrap_or(0.0);
                    }
                },
                move |err| {
                    log::error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| crate::EmulatorError::AudioError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| crate::EmulatorError::AudioError(e.to_string()))?;

        Ok((
            Self {
                _stream: stream,
                _receiver: receiver,
                _resample_buffer: Mutex::new(Vec::new()),
            },
            sender,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert!((config.sample_rate - 32040.5).abs() < 0.1);
        assert_eq!(config.channels, 2);
        assert!(config.enabled);
    }

    #[test]
    fn test_audio_batch() {
        let samples = vec![1000i16, -1000i16, 2000i16, -2000i16];
        let batch = AudioBatch::new(samples.clone(), 32040.0, 2);

        assert_eq!(batch.frame_count(), 2);
        assert_eq!(batch.samples, samples);
    }

    #[test]
    fn test_audio_batch_to_f32() {
        let samples = vec![0i16, 32767i16, -32768i16];
        let batch = AudioBatch::new(samples, 32040.0, 1);
        let f32_samples = batch.to_f32();

        assert!((f32_samples[0] - 0.0).abs() < 0.0001);
        assert!((f32_samples[1] - 0.99997).abs() < 0.0001);
        assert!((f32_samples[2] - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_audio_buffer() {
        let buffer = AudioBuffer::new();
        assert_eq!(buffer.buffer_size(), 0);

        buffer.submit_sample(1000, -1000);
        assert_eq!(buffer.buffer_size(), 2);

        let samples = buffer.drain_samples();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0], 1000);
        assert_eq!(samples[1], -1000);
    }
}
