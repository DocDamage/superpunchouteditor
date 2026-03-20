//! # Controller input handling
//!
//! Manages SNES controller state, button mapping, and input polling
//! for the emulator core.

use parking_lot::Mutex;

/// SNES controller button bitflags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnesButton(pub u16);

impl SnesButton {
    pub const B: SnesButton = SnesButton(1 << 0);
    pub const Y: SnesButton = SnesButton(1 << 1);
    pub const SELECT: SnesButton = SnesButton(1 << 2);
    pub const START: SnesButton = SnesButton(1 << 3);
    pub const UP: SnesButton = SnesButton(1 << 4);
    pub const DOWN: SnesButton = SnesButton(1 << 5);
    pub const LEFT: SnesButton = SnesButton(1 << 6);
    pub const RIGHT: SnesButton = SnesButton(1 << 7);
    pub const A: SnesButton = SnesButton(1 << 8);
    pub const X: SnesButton = SnesButton(1 << 9);
    pub const L: SnesButton = SnesButton(1 << 10);
    pub const R: SnesButton = SnesButton(1 << 11);

    /// Get the libretro button index for this button
    pub fn to_libretro_index(&self) -> libc::c_uint {
        match *self {
            SnesButton::B => crate::libretro::RETRO_DEVICE_ID_JOYPAD_B,
            SnesButton::Y => crate::libretro::RETRO_DEVICE_ID_JOYPAD_Y,
            SnesButton::SELECT => crate::libretro::RETRO_DEVICE_ID_JOYPAD_SELECT,
            SnesButton::START => crate::libretro::RETRO_DEVICE_ID_JOYPAD_START,
            SnesButton::UP => crate::libretro::RETRO_DEVICE_ID_JOYPAD_UP,
            SnesButton::DOWN => crate::libretro::RETRO_DEVICE_ID_JOYPAD_DOWN,
            SnesButton::LEFT => crate::libretro::RETRO_DEVICE_ID_JOYPAD_LEFT,
            SnesButton::RIGHT => crate::libretro::RETRO_DEVICE_ID_JOYPAD_RIGHT,
            SnesButton::A => crate::libretro::RETRO_DEVICE_ID_JOYPAD_A,
            SnesButton::X => crate::libretro::RETRO_DEVICE_ID_JOYPAD_X,
            SnesButton::L => crate::libretro::RETRO_DEVICE_ID_JOYPAD_L,
            SnesButton::R => crate::libretro::RETRO_DEVICE_ID_JOYPAD_R,
            _ => 0,
        }
    }

    /// Check if this button is pressed in a button state
    pub fn is_pressed(&self, state: u16) -> bool {
        state & self.0 != 0
    }
}

impl std::ops::BitOr for SnesButton {
    type Output = u16;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.0 | rhs.0
    }
}

impl std::ops::BitOr<u16> for SnesButton {
    type Output = u16;

    fn bitor(self, rhs: u16) -> Self::Output {
        self.0 | rhs
    }
}

/// SNES controller state
#[derive(Debug, Clone, Copy, Default)]
pub struct SnesController {
    /// Button state as bitflags
    pub buttons: u16,
    /// Left analog X (-32768 to 32767)
    pub analog_left_x: i16,
    /// Left analog Y (-32768 to 32767)
    pub analog_left_y: i16,
}

impl SnesController {
    /// Create a new controller with no buttons pressed
    pub fn new() -> Self {
        Self::default()
    }

    /// Press a button
    pub fn press(&mut self, button: SnesButton) {
        self.buttons |= button.0;
    }

    /// Release a button
    pub fn release(&mut self, button: SnesButton) {
        self.buttons &= !button.0;
    }

    /// Check if a button is pressed
    pub fn is_pressed(&self, button: SnesButton) -> bool {
        button.is_pressed(self.buttons)
    }

    /// Set the entire button state
    pub fn set_buttons(&mut self, buttons: u16) {
        self.buttons = buttons;
    }

    /// Clear all buttons
    pub fn clear(&mut self) {
        self.buttons = 0;
        self.analog_left_x = 0;
        self.analog_left_y = 0;
    }

    /// Get input state for a specific libretro button index
    pub fn get_state(&self, index: libc::c_uint) -> i16 {
        let mask = match index {
            0 => SnesButton::B.0,
            1 => SnesButton::Y.0,
            2 => SnesButton::SELECT.0,
            3 => SnesButton::START.0,
            4 => SnesButton::UP.0,
            5 => SnesButton::DOWN.0,
            6 => SnesButton::LEFT.0,
            7 => SnesButton::RIGHT.0,
            8 => SnesButton::A.0,
            9 => SnesButton::X.0,
            10 => SnesButton::L.0,
            11 => SnesButton::R.0,
            _ => 0,
        };

        if self.buttons & mask != 0 {
            1
        } else {
            0
        }
    }

    /// Get analog input for a specific axis
    pub fn get_analog(&self, index: libc::c_uint, id: libc::c_uint) -> i16 {
        match (index, id) {
            // Left analog X
            (0, 0) => self.analog_left_x,
            // Left analog Y
            (0, 1) => self.analog_left_y,
            _ => 0,
        }
    }

    /// Convert to a simple button mask (for save states)
    pub fn to_mask(&self) -> u16 {
        self.buttons
    }

    /// Load from a button mask
    pub fn from_mask(mask: u16) -> Self {
        Self {
            buttons: mask,
            analog_left_x: 0,
            analog_left_y: 0,
        }
    }
}

/// Input configuration for key/button mapping
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// Keyboard to SNES button mappings
    pub key_map: std::collections::HashMap<String, SnesButton>,
    /// Gamepad button to SNES button mappings
    pub gamepad_map: std::collections::HashMap<u32, SnesButton>,
    /// Enable analog input
    pub analog_enabled: bool,
    /// Analog sensitivity (0.0 to 2.0)
    pub analog_sensitivity: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut key_map = std::collections::HashMap::new();

        // Default keyboard mappings
        key_map.insert("ArrowUp".to_string(), SnesButton::UP);
        key_map.insert("ArrowDown".to_string(), SnesButton::DOWN);
        key_map.insert("ArrowLeft".to_string(), SnesButton::LEFT);
        key_map.insert("ArrowRight".to_string(), SnesButton::RIGHT);
        key_map.insert("z".to_string(), SnesButton::B);
        key_map.insert("a".to_string(), SnesButton::Y);
        key_map.insert("x".to_string(), SnesButton::A);
        key_map.insert("s".to_string(), SnesButton::X);
        key_map.insert("Enter".to_string(), SnesButton::START);
        key_map.insert("Shift".to_string(), SnesButton::SELECT);
        key_map.insert("q".to_string(), SnesButton::L);
        key_map.insert("w".to_string(), SnesButton::R);

        Self {
            key_map,
            gamepad_map: std::collections::HashMap::new(),
            analog_enabled: true,
            analog_sensitivity: 1.0,
        }
    }
}

impl InputConfig {
    /// Create a new input configuration with default mappings
    pub fn new() -> Self {
        Self::default()
    }

    /// Map a keyboard key to a SNES button
    pub fn map_key(&mut self, key: &str, button: SnesButton) {
        self.key_map.insert(key.to_string(), button);
    }

    /// Map a gamepad button to a SNES button
    pub fn map_gamepad(&mut self, button: u32, snes_button: SnesButton) {
        self.gamepad_map.insert(button, snes_button);
    }

    /// Get the SNES button for a keyboard key
    pub fn key_to_button(&self, key: &str) -> Option<SnesButton> {
        self.key_map.get(key).copied()
    }

    /// Get the SNES button for a gamepad button
    pub fn gamepad_to_button(&self, button: u32) -> Option<SnesButton> {
        self.gamepad_map.get(&button).copied()
    }
}

/// Thread-safe input manager
pub struct InputManager {
    /// Controller states for each port
    controllers: Vec<Mutex<SnesController>>,
    /// Input configuration
    config: parking_lot::RwLock<InputConfig>,
    /// Poll counter (for debugging/timing)
    poll_count: std::sync::atomic::AtomicU64,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self::with_port_count(2)
    }

    /// Create with specific number of ports
    pub fn with_port_count(ports: usize) -> Self {
        let mut controllers = Vec::with_capacity(ports);
        for _ in 0..ports {
            controllers.push(Mutex::new(SnesController::new()));
        }

        Self {
            controllers,
            config: parking_lot::RwLock::new(InputConfig::new()),
            poll_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Poll inputs (called by libretro)
    pub fn poll(&self) {
        self.poll_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get input state for a specific port/button
    pub fn get_state(&self, port: libc::c_uint, index: libc::c_uint, id: libc::c_uint) -> i16 {
        let port = port as usize;
        if port >= self.controllers.len() {
            return 0;
        }

        let controller = self.controllers[port].lock();

        match index {
            crate::libretro::RETRO_DEVICE_INDEX_ANALOG_LEFT => controller.get_analog(index, id),
            _ => controller.get_state(id),
        }
    }

    /// Set controller state for a port
    pub fn set_controller_state(&self, port: usize, state: SnesController) {
        if port < self.controllers.len() {
            *self.controllers[port].lock() = state;
        }
    }

    /// Get controller state for a port
    pub fn get_controller_state(&self, port: usize) -> Option<SnesController> {
        self.controllers.get(port).map(|c| *c.lock())
    }

    /// Press a button on a specific port
    pub fn press_button(&self, port: usize, button: SnesButton) {
        if port < self.controllers.len() {
            self.controllers[port].lock().press(button);
        }
    }

    /// Release a button on a specific port
    pub fn release_button(&self, port: usize, button: SnesButton) {
        if port < self.controllers.len() {
            self.controllers[port].lock().release(button);
        }
    }

    /// Clear all inputs for a port
    pub fn clear_port(&self, port: usize) {
        if port < self.controllers.len() {
            self.controllers[port].lock().clear();
        }
    }

    /// Clear all inputs
    pub fn clear_all(&self) {
        for controller in &self.controllers {
            controller.lock().clear();
        }
    }

    /// Get the poll count
    pub fn poll_count(&self) -> u64 {
        self.poll_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Set input configuration
    pub fn set_config(&self, config: InputConfig) {
        *self.config.write() = config;
    }

    /// Get input configuration
    pub fn get_config(&self) -> InputConfig {
        self.config.read().clone()
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snes_button() {
        let button = SnesButton::A;
        assert!(button.is_pressed(0x0100));
        assert!(!button.is_pressed(0x0000));
    }

    #[test]
    fn test_snes_controller() {
        let mut controller = SnesController::new();

        controller.press(SnesButton::A);
        assert!(controller.is_pressed(SnesButton::A));

        controller.release(SnesButton::A);
        assert!(!controller.is_pressed(SnesButton::A));

        controller.press(SnesButton::B);
        controller.press(SnesButton::START);
        assert_eq!(controller.buttons, 0x0109);
    }

    #[test]
    fn test_input_manager() {
        let manager = InputManager::new();

        manager.press_button(0, SnesButton::A);
        assert_eq!(manager.get_state(0, 0, 8), 1);

        manager.release_button(0, SnesButton::A);
        assert_eq!(manager.get_state(0, 0, 8), 0);
    }

    #[test]
    fn test_input_config() {
        let mut config = InputConfig::new();
        config.map_key("Space", SnesButton::START);

        assert_eq!(config.key_to_button("Space"), Some(SnesButton::START));
        assert_eq!(config.key_to_button("NonExistent"), None);
    }
}
