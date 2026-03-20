//! # libretro API bindings
//!
//! Rust FFI bindings for the libretro API based on libretro.h.
//! This module defines the types, constants, and function signatures
//! required to interface with libretro cores like Snes9x.

/// libretro API version
pub const RETRO_API_VERSION: u32 = 1;

/// Device types for input
pub const RETRO_DEVICE_NONE: libc::c_uint = 0;
pub const RETRO_DEVICE_JOYPAD: libc::c_uint = 1;
pub const RETRO_DEVICE_MOUSE: libc::c_uint = 2;
pub const RETRO_DEVICE_KEYBOARD: libc::c_uint = 3;
pub const RETRO_DEVICE_LIGHTGUN: libc::c_uint = 4;
pub const RETRO_DEVICE_ANALOG: libc::c_uint = 5;
pub const RETRO_DEVICE_POINTER: libc::c_uint = 6;

/// Joypad button indices
pub const RETRO_DEVICE_ID_JOYPAD_B: libc::c_uint = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: libc::c_uint = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: libc::c_uint = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: libc::c_uint = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: libc::c_uint = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: libc::c_uint = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: libc::c_uint = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: libc::c_uint = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: libc::c_uint = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: libc::c_uint = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: libc::c_uint = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: libc::c_uint = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: libc::c_uint = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: libc::c_uint = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: libc::c_uint = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: libc::c_uint = 15;

/// Pixel formats
pub const RETRO_PIXEL_FORMAT_0RGB1555: libc::c_uint = 0;
pub const RETRO_PIXEL_FORMAT_XRGB8888: libc::c_uint = 1;
pub const RETRO_PIXEL_FORMAT_RGB565: libc::c_uint = 2;

/// Environment commands
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: libc::c_uint = 10;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: libc::c_uint = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: libc::c_uint = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: libc::c_uint = 17;
pub const RETRO_ENVIRONMENT_GET_CAN_DUPE: libc::c_uint = 26;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS: libc::c_uint = 42;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER: libc::c_uint = 41;
pub const RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE: libc::c_uint = 23;

/// Core function type definitions
pub type RetroInitFn = unsafe extern "C" fn();
pub type RetroDeinitFn = unsafe extern "C" fn();
pub type RetroApiVersionFn = unsafe extern "C" fn() -> libc::c_uint;
pub type RetroGetSystemInfoFn = unsafe extern "C" fn(*mut RetroSystemInfo);
pub type RetroGetSystemAvInfoFn = unsafe extern "C" fn(*mut RetroSystemAvInfo);
pub type RetroSetControllerPortDeviceFn = unsafe extern "C" fn(libc::c_uint, libc::c_uint);
pub type RetroResetFn = unsafe extern "C" fn();
pub type RetroRunFn = unsafe extern "C" fn();
pub type RetroSerializeSizeFn = unsafe extern "C" fn() -> libc::size_t;
pub type RetroSerializeFn = unsafe extern "C" fn(*mut libc::c_void, libc::size_t) -> u8;
pub type RetroUnserializeFn = unsafe extern "C" fn(*const libc::c_void, libc::size_t) -> u8;
pub type RetroLoadGameFn = unsafe extern "C" fn(*const RetroGameInfo) -> u8;
pub type RetroUnloadGameFn = unsafe extern "C" fn();
pub type RetroGetMemoryDataFn = unsafe extern "C" fn(libc::c_uint) -> *mut libc::c_void;
pub type RetroGetMemorySizeFn = unsafe extern "C" fn(libc::c_uint) -> libc::size_t;

/// Callback type definitions
pub type VideoRefreshCallback =
    unsafe extern "C" fn(*const libc::c_void, libc::c_uint, libc::c_uint, libc::size_t);
pub type AudioSampleCallback = unsafe extern "C" fn(i16, i16);
pub type AudioSampleBatchCallback = unsafe extern "C" fn(*const i16, libc::size_t) -> libc::size_t;
pub type InputPollCallback = unsafe extern "C" fn();
pub type InputStateCallback =
    unsafe extern "C" fn(libc::c_uint, libc::c_uint, libc::c_uint, libc::c_uint) -> i16;
pub type EnvironmentCallback = unsafe extern "C" fn(libc::c_uint, *mut libc::c_void) -> u8;

/// Callback setter types
pub type RetroSetEnvironmentFn = unsafe extern "C" fn(EnvironmentCallback);
pub type RetroSetVideoRefreshFn = unsafe extern "C" fn(VideoRefreshCallback);
pub type RetroSetAudioSampleFn = unsafe extern "C" fn(AudioSampleCallback);
pub type RetroSetAudioSampleBatchFn = unsafe extern "C" fn(AudioSampleBatchCallback);
pub type RetroSetInputPollFn = unsafe extern "C" fn(InputPollCallback);
pub type RetroSetInputStateFn = unsafe extern "C" fn(InputStateCallback);

/// System information structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RetroSystemInfo {
    pub library_name: *const libc::c_char,
    pub library_version: *const libc::c_char,
    pub valid_extensions: *const libc::c_char,
    pub need_fullpath: u8,
    pub block_extract: u8,
}

impl Default for RetroSystemInfo {
    fn default() -> Self {
        Self {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: 0,
            block_extract: 0,
        }
    }
}

/// Game geometry information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RetroGameGeometry {
    pub base_width: libc::c_uint,
    pub base_height: libc::c_uint,
    pub max_width: libc::c_uint,
    pub max_height: libc::c_uint,
    pub aspect_ratio: libc::c_float,
}

impl Default for RetroGameGeometry {
    fn default() -> Self {
        Self {
            base_width: 256,
            base_height: 224,
            max_width: 512,
            max_height: 448,
            aspect_ratio: 4.0 / 3.0,
        }
    }
}

/// System timing information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RetroSystemTiming {
    pub fps: libc::c_double,
    pub sample_rate: libc::c_double,
}

impl Default for RetroSystemTiming {
    fn default() -> Self {
        Self {
            fps: 60.098,
            sample_rate: 32040.5,
        }
    }
}

/// Audio/Video system information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RetroSystemAvInfo {
    pub geometry: RetroGameGeometry,
    pub timing: RetroSystemTiming,
}

impl Default for RetroSystemAvInfo {
    fn default() -> Self {
        Self {
            geometry: RetroGameGeometry::default(),
            timing: RetroSystemTiming::default(),
        }
    }
}

/// Game information structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RetroGameInfo {
    pub path: *const libc::c_char,
    pub data: *const libc::c_void,
    pub size: libc::size_t,
    pub meta: *const libc::c_char,
}

impl RetroGameInfo {
    /// Create a RetroGameInfo from raw data
    pub fn from_data(data: *const u8, size: usize) -> Self {
        Self {
            path: std::ptr::null(),
            data: data as *const libc::c_void,
            size,
            meta: std::ptr::null(),
        }
    }
}

/// Variable structure for core options
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RetroVariable {
    pub key: *const libc::c_char,
    pub value: *const libc::c_char,
}

/// Memory types for get_memory_data/size
pub const RETRO_MEMORY_SAVE_RAM: libc::c_uint = 0;
pub const RETRO_MEMORY_RTC: libc::c_uint = 1;
pub const RETRO_MEMORY_SYSTEM_RAM: libc::c_uint = 2;
pub const RETRO_MEMORY_VIDEO_RAM: libc::c_uint = 3;

/// Input constants
pub const RETRO_DEVICE_INDEX_ANALOG_LEFT: libc::c_uint = 0;
pub const RETRO_DEVICE_INDEX_ANALOG_RIGHT: libc::c_uint = 1;
pub const RETRO_DEVICE_ID_ANALOG_X: libc::c_uint = 0;
pub const RETRO_DEVICE_ID_ANALOG_Y: libc::c_uint = 1;

/// Safe wrapper for converting C strings
pub unsafe fn c_str_to_string(ptr: *const libc::c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        std::ffi::CStr::from_ptr(ptr)
            .to_str()
            .ok()
            .map(|s| s.to_string())
    }
}

/// Helper to create a null-terminated C string
pub fn string_to_c_str(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).expect("CString::new failed")
}
