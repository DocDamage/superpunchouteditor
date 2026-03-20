//! # Video buffer handling
//!
//! Manages video frame buffers, pixel format conversions, and frame timing
//! for the emulator core.

use crate::{SNES_HEIGHT, SNES_MAX_HEIGHT, SNES_MAX_WIDTH, SNES_WIDTH};

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 0RGB1555 - 16-bit (default for many libretro cores)
    Format0RGB1555,
    /// XRGB8888 - 32-bit with unused alpha
    FormatXRGB8888,
    /// RGB565 - 16-bit
    FormatRGB565,
}

impl PixelFormat {
    /// Get the size in bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Format0RGB1555 => 2,
            PixelFormat::FormatRGB565 => 2,
            PixelFormat::FormatXRGB8888 => 4,
        }
    }

    /// Convert from libretro pixel format constant
    pub fn from_libretro(format: libc::c_uint) -> Option<Self> {
        match format {
            0 => Some(PixelFormat::Format0RGB1555),
            1 => Some(PixelFormat::FormatXRGB8888),
            2 => Some(PixelFormat::FormatRGB565),
            _ => None,
        }
    }

    /// Convert to libretro pixel format constant
    pub fn to_libretro(&self) -> libc::c_uint {
        match self {
            PixelFormat::Format0RGB1555 => 0,
            PixelFormat::FormatXRGB8888 => 1,
            PixelFormat::FormatRGB565 => 2,
        }
    }
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::FormatXRGB8888
    }
}

/// A single video frame from the emulator
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame buffer data (RGBA)
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Frame pitch (bytes per row)
    pub pitch: usize,
    /// Pixel format
    pub format: PixelFormat,
}

impl VideoFrame {
    /// Create a new empty video frame
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        let pitch = width as usize * format.bytes_per_pixel();
        let data = vec![0u8; pitch * height as usize];

        Self {
            data,
            width,
            height,
            pitch,
            format,
        }
    }

    /// Create a video frame from libretro callback data
    pub fn from_libretro(
        data: *const libc::c_void,
        width: libc::c_uint,
        height: libc::c_uint,
        pitch: libc::size_t,
        format: PixelFormat,
    ) -> Option<Self> {
        if data.is_null() {
            return None;
        }

        let size = pitch * height as usize;
        // # Safety
        // The pointer must be valid and point to at least `pitch * height` bytes of initialized memory.
        // The caller must ensure the data remains valid for the lifetime of this function call.
        // This is called from the libretro video callback which provides valid frame data.
        let slice = unsafe { std::slice::from_raw_parts(data as *const u8, size) };

        Some(Self {
            data: slice.to_vec(),
            width,
            height,
            pitch,
            format,
        })
    }

    /// Convert frame to RGBA format
    pub fn to_rgba(&self) -> Vec<u8> {
        match self.format {
            PixelFormat::FormatXRGB8888 => self.data.clone(),
            PixelFormat::Format0RGB1555 => self.convert_0rgb1555_to_rgba(),
            PixelFormat::FormatRGB565 => self.convert_rgb565_to_rgba(),
        }
    }

    /// Convert 0RGB1555 to RGBA
    fn convert_0rgb1555_to_rgba(&self) -> Vec<u8> {
        let pixel_count = self.data.len() / 2;
        let mut rgba = Vec::with_capacity(pixel_count * 4);

        for i in 0..pixel_count {
            let offset = i * 2;
            let pixel = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);

            let r = ((pixel >> 10) & 0x1F) as u8;
            let g = ((pixel >> 5) & 0x1F) as u8;
            let b = (pixel & 0x1F) as u8;

            // Scale 5-bit to 8-bit
            rgba.push((r << 3) | (r >> 2));
            rgba.push((g << 3) | (g >> 2));
            rgba.push((b << 3) | (b >> 2));
            rgba.push(255);
        }

        rgba
    }

    /// Convert RGB565 to RGBA
    fn convert_rgb565_to_rgba(&self) -> Vec<u8> {
        let pixel_count = self.data.len() / 2;
        let mut rgba = Vec::with_capacity(pixel_count * 4);

        for i in 0..pixel_count {
            let offset = i * 2;
            let pixel = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);

            let r = ((pixel >> 11) & 0x1F) as u8;
            let g = ((pixel >> 5) & 0x3F) as u8;
            let b = (pixel & 0x1F) as u8;

            // Scale to 8-bit
            rgba.push((r << 3) | (r >> 2));
            rgba.push((g << 2) | (g >> 4));
            rgba.push((b << 3) | (b >> 2));
            rgba.push(255);
        }

        rgba
    }

    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let offset = y as usize * self.pitch + x as usize * self.format.bytes_per_pixel();

        match self.format {
            PixelFormat::FormatXRGB8888 => {
                if offset + 3 < self.data.len() {
                    Some((
                        self.data[offset + 2],
                        self.data[offset + 1],
                        self.data[offset],
                        self.data[offset + 3],
                    ))
                } else {
                    None
                }
            }
            _ => {
                // For other formats, convert the whole frame
                let rgba = self.to_rgba();
                let idx = (y as usize * self.width as usize + x as usize) * 4;
                if idx + 3 < rgba.len() {
                    Some((rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]))
                } else {
                    None
                }
            }
        }
    }
}

/// Thread-safe video buffer manager
pub struct VideoBuffer {
    /// Current frame
    current_frame: parking_lot::Mutex<Option<VideoFrame>>,
    /// Display format
    format: parking_lot::RwLock<PixelFormat>,
    /// Frame dimensions
    width: parking_lot::RwLock<u32>,
    height: parking_lot::RwLock<u32>,
    /// Frame counter
    frame_count: std::sync::atomic::AtomicU64,
}

impl VideoBuffer {
    /// Create a new video buffer
    pub fn new() -> Self {
        Self {
            current_frame: parking_lot::Mutex::new(None),
            format: parking_lot::RwLock::new(PixelFormat::default()),
            width: parking_lot::RwLock::new(SNES_WIDTH),
            height: parking_lot::RwLock::new(SNES_HEIGHT),
            frame_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Submit a new frame from the libretro callback
    pub fn submit_frame(
        &self,
        data: *const libc::c_void,
        width: libc::c_uint,
        height: libc::c_uint,
        pitch: libc::size_t,
    ) {
        let format = *self.format.read();

        if let Some(frame) = VideoFrame::from_libretro(data, width, height, pitch, format) {
            *self.current_frame.lock() = Some(frame);
            *self.width.write() = width;
            *self.height.write() = height;
            self.frame_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Get a copy of the current frame
    pub fn get_frame(&self) -> Option<VideoFrame> {
        self.current_frame.lock().clone()
    }

    /// Get frame dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (*self.width.read(), *self.height.read())
    }

    /// Set the pixel format
    pub fn set_format(&self, format: PixelFormat) {
        *self.format.write() = format;
    }

    /// Get the current pixel format
    pub fn get_format(&self) -> PixelFormat {
        *self.format.read()
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Clear the buffer
    pub fn clear(&self) {
        *self.current_frame.lock() = None;
    }

    /// Resize buffer for new dimensions
    pub fn resize(&self, width: u32, height: u32) {
        *self.width.write() = width.min(SNES_MAX_WIDTH);
        *self.height.write() = height.min(SNES_MAX_HEIGHT);
    }
}

impl Default for VideoBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_bytes_per_pixel() {
        assert_eq!(PixelFormat::Format0RGB1555.bytes_per_pixel(), 2);
        assert_eq!(PixelFormat::FormatRGB565.bytes_per_pixel(), 2);
        assert_eq!(PixelFormat::FormatXRGB8888.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_video_frame_creation() {
        let frame = VideoFrame::new(256, 224, PixelFormat::FormatXRGB8888);
        assert_eq!(frame.width, 256);
        assert_eq!(frame.height, 224);
        assert_eq!(frame.pitch, 256 * 4);
        assert_eq!(frame.data.len(), 256 * 224 * 4);
    }

    #[test]
    fn test_video_buffer() {
        let buffer = VideoBuffer::new();
        assert_eq!(buffer.get_dimensions(), (SNES_WIDTH, SNES_HEIGHT));

        buffer.resize(512, 448);
        assert_eq!(buffer.get_dimensions(), (512, 448));
    }
}
