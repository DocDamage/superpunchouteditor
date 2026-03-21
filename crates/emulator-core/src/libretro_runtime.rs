use crate::audio::{AudioBuffer, AudioConfig};
use crate::input::InputManager;
use crate::libretro::{
    self, AudioSampleBatchCallback, AudioSampleCallback, EnvironmentCallback, InputPollCallback,
    InputStateCallback, RetroApiVersionFn, RetroDeinitFn, RetroGameInfo,
    RetroGetMemoryDataFn, RetroGetMemorySizeFn, RetroGetSystemAvInfoFn, RetroGetSystemInfoFn,
    RetroInitFn, RetroLoadGameFn, RetroResetFn, RetroRunFn, RetroSerializeFn,
    RetroSerializeSizeFn, RetroSetAudioSampleBatchFn, RetroSetAudioSampleFn,
    RetroSetControllerPortDeviceFn, RetroSetEnvironmentFn, RetroSetInputPollFn,
    RetroSetInputStateFn, RetroSetVideoRefreshFn, RetroSystemAvInfo, RetroSystemInfo,
    RetroUnloadGameFn, RetroUnserializeFn, VideoRefreshCallback,
};
use crate::video::{PixelFormat, VideoBuffer};
use crate::{EmulatorError, Result};
use libloading::Library;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default)]
struct CallbackTargets {
    video: Option<Arc<VideoBuffer>>,
    audio: Option<Arc<AudioBuffer>>,
    input: Option<Arc<InputManager>>,
}

static CALLBACK_TARGETS: Lazy<Mutex<CallbackTargets>> =
    Lazy::new(|| Mutex::new(CallbackTargets::default()));

fn register_callback_targets(
    video: &Arc<VideoBuffer>,
    audio: &Arc<AudioBuffer>,
    input: &Arc<InputManager>,
) {
    let mut targets = CALLBACK_TARGETS.lock();
    targets.video = Some(video.clone());
    targets.audio = Some(audio.clone());
    targets.input = Some(input.clone());
}

pub fn clear_callback_targets() {
    *CALLBACK_TARGETS.lock() = CallbackTargets::default();
}

unsafe extern "C" fn environment_callback(
    cmd: libc::c_uint,
    data: *mut libc::c_void,
) -> u8 {
    match cmd {
        libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            if data.is_null() {
                return 0;
            }

            let format = unsafe { *(data as *const libc::c_uint) };
            let Some(pixel_format) = PixelFormat::from_libretro(format) else {
                return 0;
            };

            let video = { CALLBACK_TARGETS.lock().video.clone() };
            if let Some(video) = video {
                video.set_format(pixel_format);
            }
            1
        }
        libretro::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
            if !data.is_null() {
                unsafe { *(data as *mut bool) = true };
            }
            1
        }
        libretro::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
            if !data.is_null() {
                unsafe { *(data as *mut bool) = false };
            }
            1
        }
        libretro::RETRO_ENVIRONMENT_SET_VARIABLES
        | libretro::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS => 1,
        _ => 0,
    }
}

unsafe extern "C" fn video_refresh_callback(
    data: *const libc::c_void,
    width: libc::c_uint,
    height: libc::c_uint,
    pitch: libc::size_t,
) {
    let video = { CALLBACK_TARGETS.lock().video.clone() };
    if let Some(video) = video {
        video.submit_frame(data, width, height, pitch);
    }
}

unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    let audio = { CALLBACK_TARGETS.lock().audio.clone() };
    if let Some(audio) = audio {
        audio.submit_sample(left, right);
    }
}

unsafe extern "C" fn audio_sample_batch_callback(
    data: *const i16,
    frames: libc::size_t,
) -> libc::size_t {
    let audio = { CALLBACK_TARGETS.lock().audio.clone() };
    if let Some(audio) = audio {
        return audio.submit_batch(data, frames);
    }
    0
}

unsafe extern "C" fn input_poll_callback() {
    let input = { CALLBACK_TARGETS.lock().input.clone() };
    if let Some(input) = input {
        input.poll();
    }
}

unsafe extern "C" fn input_state_callback(
    port: libc::c_uint,
    device: libc::c_uint,
    index: libc::c_uint,
    id: libc::c_uint,
) -> i16 {
    if device != libretro::RETRO_DEVICE_JOYPAD && device != libretro::RETRO_DEVICE_ANALOG {
        return 0;
    }

    let input = { CALLBACK_TARGETS.lock().input.clone() };
    if let Some(input) = input {
        return input.get_state(port, index, id);
    }
    0
}

fn load_symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T> {
    unsafe {
        library
            .get::<T>(name)
            .map(|symbol| *symbol)
            .map_err(|error| EmulatorError::LibraryLoadError(error.to_string()))
    }
}

pub struct LibretroCore {
    _library: Library,
    pub needs_fullpath: bool,
    retro_deinit: RetroDeinitFn,
    retro_set_controller_port_device: RetroSetControllerPortDeviceFn,
    retro_reset: RetroResetFn,
    retro_run: RetroRunFn,
    retro_serialize_size: RetroSerializeSizeFn,
    retro_serialize: RetroSerializeFn,
    retro_unserialize: RetroUnserializeFn,
    retro_load_game: RetroLoadGameFn,
    retro_unload_game: RetroUnloadGameFn,
    retro_get_memory_data: RetroGetMemoryDataFn,
    retro_get_memory_size: RetroGetMemorySizeFn,
    retro_get_system_av_info: RetroGetSystemAvInfoFn,
    game_loaded: bool,
}

impl LibretroCore {
    pub fn load(
        core_path: &str,
        video_buffer: &Arc<VideoBuffer>,
        audio_buffer: &Arc<AudioBuffer>,
        input_manager: &Arc<InputManager>,
    ) -> Result<Self> {
        register_callback_targets(video_buffer, audio_buffer, input_manager);

        let library = unsafe {
            Library::new(core_path)
                .map_err(|error| EmulatorError::LibraryLoadError(error.to_string()))?
        };

        let retro_set_environment: RetroSetEnvironmentFn =
            load_symbol(&library, b"retro_set_environment\0")?;
        let retro_set_video_refresh: RetroSetVideoRefreshFn =
            load_symbol(&library, b"retro_set_video_refresh\0")?;
        let retro_set_audio_sample: RetroSetAudioSampleFn =
            load_symbol(&library, b"retro_set_audio_sample\0")?;
        let retro_set_audio_sample_batch: RetroSetAudioSampleBatchFn =
            load_symbol(&library, b"retro_set_audio_sample_batch\0")?;
        let retro_set_input_poll: RetroSetInputPollFn =
            load_symbol(&library, b"retro_set_input_poll\0")?;
        let retro_set_input_state: RetroSetInputStateFn =
            load_symbol(&library, b"retro_set_input_state\0")?;
        let retro_init: RetroInitFn = load_symbol(&library, b"retro_init\0")?;
        let retro_deinit: RetroDeinitFn = load_symbol(&library, b"retro_deinit\0")?;
        let retro_api_version: RetroApiVersionFn = load_symbol(&library, b"retro_api_version\0")?;
        let retro_get_system_info: RetroGetSystemInfoFn =
            load_symbol(&library, b"retro_get_system_info\0")?;
        let retro_get_system_av_info: RetroGetSystemAvInfoFn =
            load_symbol(&library, b"retro_get_system_av_info\0")?;
        let retro_set_controller_port_device: RetroSetControllerPortDeviceFn =
            load_symbol(&library, b"retro_set_controller_port_device\0")?;
        let retro_reset: RetroResetFn = load_symbol(&library, b"retro_reset\0")?;
        let retro_run: RetroRunFn = load_symbol(&library, b"retro_run\0")?;
        let retro_serialize_size: RetroSerializeSizeFn =
            load_symbol(&library, b"retro_serialize_size\0")?;
        let retro_serialize: RetroSerializeFn = load_symbol(&library, b"retro_serialize\0")?;
        let retro_unserialize: RetroUnserializeFn =
            load_symbol(&library, b"retro_unserialize\0")?;
        let retro_load_game: RetroLoadGameFn = load_symbol(&library, b"retro_load_game\0")?;
        let retro_unload_game: RetroUnloadGameFn = load_symbol(&library, b"retro_unload_game\0")?;
        let retro_get_memory_data: RetroGetMemoryDataFn =
            load_symbol(&library, b"retro_get_memory_data\0")?;
        let retro_get_memory_size: RetroGetMemorySizeFn =
            load_symbol(&library, b"retro_get_memory_size\0")?;

        unsafe {
            retro_set_environment(environment_callback as EnvironmentCallback);
            retro_set_video_refresh(video_refresh_callback as VideoRefreshCallback);
            retro_set_audio_sample(audio_sample_callback as AudioSampleCallback);
            retro_set_audio_sample_batch(
                audio_sample_batch_callback as AudioSampleBatchCallback,
            );
            retro_set_input_poll(input_poll_callback as InputPollCallback);
            retro_set_input_state(input_state_callback as InputStateCallback);
            retro_init();
        }

        let api_version = unsafe { retro_api_version() };
        if api_version != libretro::RETRO_API_VERSION {
            clear_callback_targets();
            return Err(EmulatorError::InitializationError(format!(
                "Unsupported libretro API version {api_version}"
            )));
        }

        let mut system_info = RetroSystemInfo::default();
        unsafe { retro_get_system_info(&mut system_info) };

        Ok(Self {
            _library: library,
            needs_fullpath: system_info.need_fullpath != 0,
            retro_deinit,
            retro_set_controller_port_device,
            retro_reset,
            retro_run,
            retro_serialize_size,
            retro_serialize,
            retro_unserialize,
            retro_load_game,
            retro_unload_game,
            retro_get_memory_data,
            retro_get_memory_size,
            retro_get_system_av_info,
            game_loaded: false,
        })
    }

    pub fn load_rom(
        &mut self,
        rom_data: &[u8],
        video_buffer: &Arc<VideoBuffer>,
        audio_buffer: &Arc<AudioBuffer>,
    ) -> Result<()> {
        if self.needs_fullpath {
            return Err(EmulatorError::RomLoadError(
                "The configured libretro core requires full-path ROM loading".to_string(),
            ));
        }

        if self.game_loaded {
            self.unload_game();
        }

        let game_info = RetroGameInfo::from_data(rom_data.as_ptr(), rom_data.len());
        let loaded = unsafe { (self.retro_load_game)(&game_info) };
        if loaded == 0 {
            return Err(EmulatorError::RomLoadError(
                "libretro core rejected the ROM payload".to_string(),
            ));
        }

        self.game_loaded = true;

        unsafe {
            (self.retro_set_controller_port_device)(0, libretro::RETRO_DEVICE_JOYPAD);
            (self.retro_set_controller_port_device)(1, libretro::RETRO_DEVICE_NONE);
        }

        let mut av_info = RetroSystemAvInfo::default();
        unsafe { (self.retro_get_system_av_info)(&mut av_info) };
        video_buffer.resize(av_info.geometry.base_width, av_info.geometry.base_height);
        Self::set_audio_config(audio_buffer, &av_info);

        Ok(())
    }

    pub fn run_frame(&self) {
        unsafe { (self.retro_run)() };
    }

    pub fn reset(&self) {
        unsafe { (self.retro_reset)() };
    }

    pub fn save_state(&self) -> Result<Vec<u8>> {
        let size = unsafe { (self.retro_serialize_size)() };
        if size == 0 {
            return Err(EmulatorError::StateError(
                "libretro core does not expose serialization".to_string(),
            ));
        }

        let mut buffer = vec![0u8; size];
        let ok = unsafe { (self.retro_serialize)(buffer.as_mut_ptr() as *mut libc::c_void, size) };
        if ok == 0 {
            return Err(EmulatorError::StateError(
                "libretro serialize call failed".to_string(),
            ));
        }

        Ok(buffer)
    }

    pub fn load_state(&self, state_data: &[u8]) -> Result<()> {
        let ok = unsafe {
            (self.retro_unserialize)(
                state_data.as_ptr() as *const libc::c_void,
                state_data.len(),
            )
        };
        if ok == 0 {
            return Err(EmulatorError::StateError(
                "libretro unserialize call failed".to_string(),
            ));
        }
        Ok(())
    }

    pub fn save_state_size(&self) -> usize {
        unsafe { (self.retro_serialize_size)() }
    }

    pub fn read_memory(
        &self,
        memory_id: libc::c_uint,
        offset: usize,
        len: usize,
    ) -> Option<Vec<u8>> {
        let end = offset.checked_add(len)?;
        let size = unsafe { (self.retro_get_memory_size)(memory_id) };
        if size == 0 || end > size {
            return None;
        }

        let data = unsafe { (self.retro_get_memory_data)(memory_id) } as *const u8;
        if data.is_null() {
            return None;
        }

        let slice = unsafe { std::slice::from_raw_parts(data, size) };
        Some(slice[offset..end].to_vec())
    }

    pub fn write_memory(
        &self,
        memory_id: libc::c_uint,
        offset: usize,
        bytes: &[u8],
    ) -> bool {
        let end = match offset.checked_add(bytes.len()) {
            Some(end) => end,
            None => return false,
        };
        let size = unsafe { (self.retro_get_memory_size)(memory_id) };
        if size == 0 || end > size {
            return false;
        }

        let data = unsafe { (self.retro_get_memory_data)(memory_id) } as *mut u8;
        if data.is_null() {
            return false;
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(data, size) };
        slice[offset..end].copy_from_slice(bytes);
        true
    }

    pub fn unload_game(&mut self) {
        if self.game_loaded {
            unsafe { (self.retro_unload_game)() };
            self.game_loaded = false;
        }
    }

    pub fn set_audio_config(audio_buffer: &Arc<AudioBuffer>, av_info: &RetroSystemAvInfo) {
        audio_buffer.set_config(AudioConfig::default().with_sample_rate(av_info.timing.sample_rate));
    }
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        self.unload_game();
        unsafe { (self.retro_deinit)() };
    }
}
