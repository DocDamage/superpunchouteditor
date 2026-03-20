//! SPC700 Audio Debugger
//!
//! Provides debugging capabilities for the SPC700 audio coprocessor,
//! including DSP register inspection and audio channel state monitoring.

use serde::{Deserialize, Serialize};

/// SPC700 Audio RAM size (64KB)
pub const SPC700_RAM_SIZE: usize = 0x10000;

/// Number of audio channels
pub const SPC700_CHANNELS: usize = 8;

/// DSP register addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DspRegister {
    /// Left channel master volume
    MasterVolLeft = 0x0C,
    /// Right channel master volume
    MasterVolRight = 0x1C,
    /// Echo left volume
    EchoVolLeft = 0x2C,
    /// Echo right volume
    EchoVolRight = 0x3C,
    /// Key on (start channels)
    KeyOn = 0x4C,
    /// Key off (stop channels)
    KeyOff = 0x5C,
    /// Flags and reset
    Flags = 0x6C,
    /// End and block (sample end bits)
    EndX = 0x7C,
    /// Echo feedback
    EchoFeedback = 0x0D,
    /// Sample source directory offset
    SourceDir = 0x5D,
    /// Echo buffer start offset
    EchoStart = 0x6D,
    /// Echo delay
    EchoDelay = 0x7D,
    /// Pitch modulation
    PitchMod = 0x2D,
    /// Noise enable
    NoiseEnable = 0x3D,
    /// Echo enable
    EchoEnable = 0x4D,
}

impl DspRegister {
    /// Get the register address for a channel-specific register
    pub fn for_channel(self, channel: usize) -> u8 {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self as u8) | (channel as u8)
    }

    /// Get all register addresses
    pub fn all() -> &'static [DspRegister] {
        &[
            DspRegister::MasterVolLeft,
            DspRegister::MasterVolRight,
            DspRegister::EchoVolLeft,
            DspRegister::EchoVolRight,
            DspRegister::KeyOn,
            DspRegister::KeyOff,
            DspRegister::Flags,
            DspRegister::EndX,
            DspRegister::EchoFeedback,
            DspRegister::SourceDir,
            DspRegister::EchoStart,
            DspRegister::EchoDelay,
            DspRegister::PitchMod,
            DspRegister::NoiseEnable,
            DspRegister::EchoEnable,
        ]
    }
}

impl std::fmt::Display for DspRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DspRegister::MasterVolLeft => write!(f, "MVOL(L)"),
            DspRegister::MasterVolRight => write!(f, "MVOL(R)"),
            DspRegister::EchoVolLeft => write!(f, "EVOL(L)"),
            DspRegister::EchoVolRight => write!(f, "EVOL(R)"),
            DspRegister::KeyOn => write!(f, "KON"),
            DspRegister::KeyOff => write!(f, "KOF"),
            DspRegister::Flags => write!(f, "FLG"),
            DspRegister::EndX => write!(f, "ENDX"),
            DspRegister::EchoFeedback => write!(f, "EFB"),
            DspRegister::SourceDir => write!(f, "DIR"),
            DspRegister::EchoStart => write!(f, "ESA"),
            DspRegister::EchoDelay => write!(f, "EDL"),
            DspRegister::PitchMod => write!(f, "PMON"),
            DspRegister::NoiseEnable => write!(f, "NON"),
            DspRegister::EchoEnable => write!(f, "EON"),
        }
    }
}

/// Channel-specific DSP register addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelRegister {
    /// Left channel volume
    VolLeft = 0x00,
    /// Right channel volume
    VolRight = 0x01,
    /// Pitch low byte
    PitchLow = 0x02,
    /// Pitch high byte
    PitchHigh = 0x03,
    /// Sample source number
    SourceNumber = 0x04,
    /// ADSR envelope settings (1st byte)
    Adsr1 = 0x05,
    /// ADSR envelope settings (2nd byte)
    Adsr2 = 0x06,
    /// Gain/envelope value
    Gain = 0x07,
    /// Current envelope value (read-only)
    EnvX = 0x08,
    /// Current sample value (read-only)
    OutX = 0x09,
}

impl ChannelRegister {
    /// Get the DSP register address for this register on a specific channel
    pub fn address(self, channel: usize) -> u8 {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self as u8) | ((channel as u8) << 4)
    }
}

/// State of a single audio channel
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioChannelState {
    /// Channel number (0-7)
    pub channel: usize,
    /// Left channel volume (-128 to 127)
    pub vol_left: i8,
    /// Right channel volume (-128 to 127)
    pub vol_right: i8,
    /// Pitch (14-bit value, 0-16383)
    pub pitch: u16,
    /// Sample source number (0-255)
    pub source_number: u8,
    /// ADSR envelope settings (byte 1)
    pub adsr1: u8,
    /// ADSR envelope settings (byte 2)
    pub adsr2: u8,
    /// Gain/envelope setting
    pub gain: u8,
    /// Current envelope value (read-only from DSP)
    pub envx: u8,
    /// Current sample output value (read-only from DSP)
    pub outx: i8,
    /// Channel is currently playing
    pub is_playing: bool,
    /// Channel has reached sample end
    pub sample_ended: bool,
}

impl AudioChannelState {
    /// Create a new channel state for the given channel number
    pub fn new(channel: usize) -> Self {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        Self {
            channel,
            ..Default::default()
        }
    }

    /// Get the attack rate from ADSR1
    pub fn attack_rate(&self) -> u8 {
        self.adsr1 & 0x0F
    }

    /// Get the decay rate from ADSR1
    pub fn decay_rate(&self) -> u8 {
        (self.adsr1 >> 4) & 0x07
    }

    /// Check if ADSR mode is enabled
    pub fn adsr_enabled(&self) -> bool {
        (self.adsr1 & 0x80) != 0
    }

    /// Get the sustain level from ADSR2
    pub fn sustain_level(&self) -> u8 {
        (self.adsr2 >> 5) & 0x07
    }

    /// Get the release rate from ADSR2
    pub fn release_rate(&self) -> u8 {
        self.adsr2 & 0x1F
    }

    /// Check if gain is in direct mode
    pub fn gain_direct_mode(&self) -> bool {
        (self.gain & 0x80) == 0
    }

    /// Get direct gain value
    pub fn direct_gain(&self) -> u8 {
        self.gain & 0x7F
    }

    /// Get the frequency in Hz (approximate)
    /// pitch value * 32000 Hz / 2^12
    pub fn frequency_hz(&self) -> f32 {
        (self.pitch as f32 * 32000.0) / 4096.0
    }

    /// Get volume as a float (-1.0 to 1.0)
    pub fn normalized_vol_left(&self) -> f32 {
        self.vol_left as f32 / 128.0
    }

    /// Get volume as a float (-1.0 to 1.0)
    pub fn normalized_vol_right(&self) -> f32 {
        self.vol_right as f32 / 128.0
    }
}

/// Complete DSP register state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DspRegisterState {
    /// Master volume left (-128 to 127)
    pub master_vol_left: i8,
    /// Master volume right (-128 to 127)
    pub master_vol_right: i8,
    /// Echo volume left (-128 to 127)
    pub echo_vol_left: i8,
    /// Echo volume right (-128 to 127)
    pub echo_vol_right: i8,
    /// Key on register (bits 0-7 for channels 0-7)
    pub key_on: u8,
    /// Key off register (bits 0-7 for channels 0-7)
    pub key_off: u8,
    /// Flags: reset, mute, echo write disable, noise clock
    pub flags: u8,
    /// End block (sample ended bits)
    pub endx: u8,
    /// Echo feedback (-128 to 127)
    pub echo_feedback: i8,
    /// Sample source directory offset (in 0x100 byte pages)
    pub source_dir: u8,
    /// Echo buffer start offset (in 0x100 byte pages)
    pub echo_start: u8,
    /// Echo delay (0-15, multiply by 0x800 for buffer size)
    pub echo_delay: u8,
    /// Pitch modulation enable (bit flags)
    pub pitch_mod: u8,
    /// Noise enable (bit flags)
    pub noise_enable: u8,
    /// Echo enable (bit flags)
    pub echo_enable: u8,
}

impl DspRegisterState {
    /// Check if reset bit is set
    pub fn is_reset(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    /// Check if mute bit is set
    pub fn is_muted(&self) -> bool {
        (self.flags & 0x40) != 0
    }

    /// Check if echo write is disabled
    pub fn echo_write_disabled(&self) -> bool {
        (self.flags & 0x20) != 0
    }

    /// Get noise clock frequency setting (0-31)
    pub fn noise_clock(&self) -> u8 {
        self.flags & 0x1F
    }

    /// Check if a channel is keyed on
    pub fn channel_active(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.key_on & (1 << channel)) != 0
    }

    /// Check if a channel is being keyed off
    pub fn channel_releasing(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.key_off & (1 << channel)) != 0
    }

    /// Check if a channel has ended
    pub fn channel_ended(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.endx & (1 << channel)) != 0
    }

    /// Check if pitch modulation is enabled for a channel
    pub fn pitch_mod_enabled(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.pitch_mod & (1 << channel)) != 0
    }

    /// Check if noise is enabled for a channel
    pub fn noise_enabled(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.noise_enable & (1 << channel)) != 0
    }

    /// Check if echo is enabled for a channel
    pub fn echo_enabled(&self, channel: usize) -> bool {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");
        (self.echo_enable & (1 << channel)) != 0
    }

    /// Get the sample source directory address
    pub fn source_dir_address(&self) -> u16 {
        (self.source_dir as u16) << 8
    }

    /// Get the echo buffer start address
    pub fn echo_start_address(&self) -> u16 {
        (self.echo_start as u16) << 8
    }

    /// Get the echo buffer size
    pub fn echo_buffer_size(&self) -> usize {
        (self.echo_delay as usize + 1) * 0x800
    }
}

/// SPC700 memory and register state
#[derive(Debug, Clone)]
pub struct Spc700State {
    /// 64KB of audio RAM
    pub ram: Vec<u8>,
    /// DSP register mirror (128 bytes)
    pub dsp_registers: [u8; 128],
    /// Program counter
    pub pc: u16,
    /// Accumulator (A)
    pub a: u8,
    /// X register
    pub x: u8,
    /// Y register
    pub y: u8,
    /// Stack pointer
    pub sp: u8,
    /// Processor status (P)
    pub p: u8,
    /// Current DSP register address
    pub dsp_addr: u8,
    /// I/O ports (4 ports for communication with main CPU)
    pub ports: [u8; 4],
}

impl Default for Spc700State {
    fn default() -> Self {
        Self::new()
    }
}

impl Spc700State {
    /// Create a new SPC700 state
    pub fn new() -> Self {
        Self {
            ram: vec![0; SPC700_RAM_SIZE],
            dsp_registers: [0; 128],
            pc: 0xFFC0,
            a: 0,
            x: 0,
            y: 0,
            sp: 0xEF,
            p: 0x02,
            dsp_addr: 0,
            ports: [0; 4],
        }
    }

    /// Read a byte from RAM
    pub fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    /// Write a byte to RAM
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    /// Read a DSP register
    pub fn read_dsp(&self, reg: u8) -> u8 {
        self.dsp_registers[reg as usize]
    }

    /// Write a DSP register
    pub fn write_dsp(&mut self, reg: u8, value: u8) {
        self.dsp_registers[reg as usize] = value;
    }

    /// Set the DSP register address
    pub fn set_dsp_addr(&mut self, addr: u8) {
        self.dsp_addr = addr;
    }

    /// Read from the currently selected DSP register
    pub fn read_dsp_current(&self) -> u8 {
        self.read_dsp(self.dsp_addr)
    }

    /// Write to the currently selected DSP register
    pub fn write_dsp_current(&mut self, value: u8) {
        self.write_dsp(self.dsp_addr, value);
    }

    /// Get the DSP register state
    pub fn dsp_state(&self) -> DspRegisterState {
        DspRegisterState {
            master_vol_left: self.dsp_registers[DspRegister::MasterVolLeft as usize] as i8,
            master_vol_right: self.dsp_registers[DspRegister::MasterVolRight as usize] as i8,
            echo_vol_left: self.dsp_registers[DspRegister::EchoVolLeft as usize] as i8,
            echo_vol_right: self.dsp_registers[DspRegister::EchoVolRight as usize] as i8,
            key_on: self.dsp_registers[DspRegister::KeyOn as usize],
            key_off: self.dsp_registers[DspRegister::KeyOff as usize],
            flags: self.dsp_registers[DspRegister::Flags as usize],
            endx: self.dsp_registers[DspRegister::EndX as usize],
            echo_feedback: self.dsp_registers[DspRegister::EchoFeedback as usize] as i8,
            source_dir: self.dsp_registers[DspRegister::SourceDir as usize],
            echo_start: self.dsp_registers[DspRegister::EchoStart as usize],
            echo_delay: self.dsp_registers[DspRegister::EchoDelay as usize],
            pitch_mod: self.dsp_registers[DspRegister::PitchMod as usize],
            noise_enable: self.dsp_registers[DspRegister::NoiseEnable as usize],
            echo_enable: self.dsp_registers[DspRegister::EchoEnable as usize],
        }
    }

    /// Get state for a specific channel
    pub fn channel_state(&self, channel: usize) -> AudioChannelState {
        assert!(channel < SPC700_CHANNELS, "Invalid channel number");

        let dsp_state = self.dsp_state();

        AudioChannelState {
            channel,
            vol_left: self.dsp_registers[ChannelRegister::VolLeft.address(channel) as usize] as i8,
            vol_right: self.dsp_registers[ChannelRegister::VolRight.address(channel) as usize] as i8,
            pitch: {
                let low = self.dsp_registers[ChannelRegister::PitchLow.address(channel) as usize] as u16;
                let high = self.dsp_registers[ChannelRegister::PitchHigh.address(channel) as usize] as u16;
                (high << 8) | low
            },
            source_number: self.dsp_registers[ChannelRegister::SourceNumber.address(channel) as usize],
            adsr1: self.dsp_registers[ChannelRegister::Adsr1.address(channel) as usize],
            adsr2: self.dsp_registers[ChannelRegister::Adsr2.address(channel) as usize],
            gain: self.dsp_registers[ChannelRegister::Gain.address(channel) as usize],
            envx: self.dsp_registers[ChannelRegister::EnvX.address(channel) as usize],
            outx: self.dsp_registers[ChannelRegister::OutX.address(channel) as usize] as i8,
            is_playing: dsp_state.channel_active(channel),
            sample_ended: dsp_state.channel_ended(channel),
        }
    }

    /// Get all channel states
    pub fn all_channel_states(&self) -> Vec<AudioChannelState> {
        (0..SPC700_CHANNELS)
            .map(|ch| self.channel_state(ch))
            .collect()
    }
}

/// SPC700 debugger
#[derive(Debug)]
pub struct Spc700Debugger {
    /// Current SPC700 state
    state: Spc700State,
    /// Breakpoints on SPC700 addresses
    breakpoints: Vec<u16>,
    /// Break on DSP register writes
    dsp_write_breakpoints: Vec<u8>,
    /// Trace log of recent operations
    trace_log: Vec<Spc700TraceEntry>,
    /// Maximum trace log size (currently unused)
    _max_trace_size: usize,
    /// Enabled/disabled
    enabled: bool,
    /// Cycle counter
    cycles: u64,
}

impl Default for Spc700Debugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Spc700Debugger {
    /// Create a new SPC700 debugger
    pub fn new() -> Self {
        Self {
            state: Spc700State::new(),
            breakpoints: Vec::new(),
            dsp_write_breakpoints: Vec::new(),
            trace_log: Vec::with_capacity(1000),
            _max_trace_size: 1000,
            enabled: true,
            cycles: 0,
        }
    }

    /// Enable/disable the debugger
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if debugger is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a reference to the SPC700 state
    pub fn state(&self) -> &Spc700State {
        &self.state
    }

    /// Get a mutable reference to the SPC700 state
    pub fn state_mut(&mut self) -> &mut Spc700State {
        &mut self.state
    }

    /// Read SPC700 memory
    pub fn read_memory(&self, addr: u16, size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            result.push(self.state.read_ram(addr.wrapping_add(i as u16)));
        }
        result
    }

    /// Write SPC700 memory
    pub fn write_memory(&mut self, addr: u16, data: &[u8]) {
        for (i, &value) in data.iter().enumerate() {
            self.state.write_ram(addr.wrapping_add(i as u16), value);
        }
    }

    /// Read a DSP register
    pub fn read_dsp_register(&self, reg: u8) -> u8 {
        self.state.read_dsp(reg)
    }

    /// Write a DSP register
    pub fn write_dsp_register(&mut self, reg: u8, value: u8) {
        self.state.write_dsp(reg, value);

        // Check for breakpoint
        if self.dsp_write_breakpoints.contains(&reg) {
            // Would trigger breakpoint here
        }
    }

    /// Get the current DSP register state
    pub fn dsp_state(&self) -> DspRegisterState {
        self.state.dsp_state()
    }

    /// Get the state of a specific audio channel
    pub fn channel_state(&self, channel: usize) -> AudioChannelState {
        self.state.channel_state(channel)
    }

    /// Get all channel states
    pub fn all_channels(&self) -> Vec<AudioChannelState> {
        self.state.all_channel_states()
    }

    /// Get currently playing channels
    pub fn active_channels(&self) -> Vec<AudioChannelState> {
        self.all_channels()
            .into_iter()
            .filter(|ch| ch.is_playing)
            .collect()
    }

    /// Add a breakpoint at an SPC700 address
    pub fn add_breakpoint(&mut self, addr: u16) {
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
        }
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, addr: u16) {
        self.breakpoints.retain(|&a| a != addr);
    }

    /// Add a DSP write breakpoint
    pub fn add_dsp_breakpoint(&mut self, reg: u8) {
        if !self.dsp_write_breakpoints.contains(&reg) {
            self.dsp_write_breakpoints.push(reg);
        }
    }

    /// Remove a DSP write breakpoint
    pub fn remove_dsp_breakpoint(&mut self, reg: u8) {
        self.dsp_write_breakpoints.retain(|&r| r != reg);
    }

    /// Get sample information from the source directory
    /// Each entry is 4 bytes: start(2), loop(2)
    pub fn get_sample_info(&self, sample_num: u8) -> Option<SampleInfo> {
        let dir_addr = self.state.dsp_state().source_dir_address();
        let entry_addr = dir_addr + (sample_num as u16) * 4;

        let start_low = self.state.read_ram(entry_addr) as u16;
        let start_high = self.state.read_ram(entry_addr + 1) as u16;
        let loop_low = self.state.read_ram(entry_addr + 2) as u16;
        let loop_high = self.state.read_ram(entry_addr + 3) as u16;

        Some(SampleInfo {
            sample_number: sample_num,
            start_address: (start_high << 8) | start_low,
            loop_address: (loop_high << 8) | loop_low,
        })
    }

    /// Clear the trace log
    pub fn clear_trace(&mut self) {
        self.trace_log.clear();
    }

    /// Get the trace log
    pub fn trace_log(&self) -> &[Spc700TraceEntry] {
        &self.trace_log
    }

    /// Step the SPC700 one instruction (placeholder)
    pub fn step(&mut self) {
        self.cycles += 1;
        // Actual implementation would decode and execute one instruction
    }

    /// Reset the SPC700
    pub fn reset(&mut self) {
        self.state = Spc700State::new();
        self.cycles = 0;
        self.clear_trace();
    }

    /// Get the sample data for a BRR block
    /// Returns the 9 bytes of the BRR block header + samples
    pub fn read_brr_block(&self, addr: u16) -> [u8; 9] {
        let mut block = [0u8; 9];
        for i in 0..9 {
            block[i] = self.state.read_ram(addr.wrapping_add(i as u16));
        }
        block
    }
}

/// Information about a BRR sample
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SampleInfo {
    /// Sample number in the directory
    pub sample_number: u8,
    /// Starting address in SPC700 RAM
    pub start_address: u16,
    /// Loop point address in SPC700 RAM
    pub loop_address: u16,
}

/// A single trace entry for SPC700 execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spc700TraceEntry {
    /// Program counter
    pub pc: u16,
    /// Accumulator
    pub a: u8,
    /// X register
    pub x: u8,
    /// Y register
    pub y: u8,
    /// Stack pointer
    pub sp: u8,
    /// Processor status
    pub p: u8,
    /// Opcode
    pub opcode: u8,
    /// Disassembled instruction
    pub disassembly: String,
    /// Cycle count
    pub cycle: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_register_state() {
        let mut state = Spc700State::new();
        
        // Set up some DSP registers
        state.write_dsp(DspRegister::MasterVolLeft as u8, 0x7F);
        state.write_dsp(DspRegister::KeyOn as u8, 0x03); // Channels 0 and 1
        state.write_dsp(DspRegister::Flags as u8, 0xC0); // Reset + Mute
        
        let dsp = state.dsp_state();
        
        assert_eq!(dsp.master_vol_left, 0x7F);
        assert!(dsp.channel_active(0));
        assert!(dsp.channel_active(1));
        assert!(!dsp.channel_active(2));
        assert!(dsp.is_reset());
        assert!(dsp.is_muted());
    }

    #[test]
    fn test_audio_channel_state() {
        let mut state = Spc700State::new();
        
        // Set up channel 0
        state.write_dsp(ChannelRegister::VolLeft.address(0), 0x40);
        state.write_dsp(ChannelRegister::VolRight.address(0), 0x60);
        state.write_dsp(ChannelRegister::PitchLow.address(0), 0x34);
        state.write_dsp(ChannelRegister::PitchHigh.address(0), 0x12);
        state.write_dsp(ChannelRegister::Adsr1.address(0), 0x8F);
        state.write_dsp(ChannelRegister::Adsr2.address(0), 0xE0);
        
        let ch = state.channel_state(0);
        
        assert_eq!(ch.vol_left, 0x40);
        assert_eq!(ch.vol_right, 0x60);
        assert_eq!(ch.pitch, 0x1234);
        assert_eq!(ch.attack_rate(), 0x0F);
        assert_eq!(ch.decay_rate(), 0x00);
        assert!(ch.adsr_enabled());
        assert_eq!(ch.sustain_level(), 0x07);
    }

    #[test]
    fn test_spc700_memory() {
        let mut state = Spc700State::new();
        
        state.write_ram(0x1000, 0x42);
        assert_eq!(state.read_ram(0x1000), 0x42);
        
        // Test wrap-around
        state.write_ram(0xFFFF, 0x55);
        assert_eq!(state.read_ram(0xFFFF), 0x55);
    }

    #[test]
    fn test_sample_info() {
        let mut debugger = Spc700Debugger::new();
        
        // Set up source directory at page 0x02
        debugger.state.write_dsp(DspRegister::SourceDir as u8, 0x02);
        
        // Sample 0 entry at 0x0200: start=$1234, loop=$5678
        debugger.state.write_ram(0x0200, 0x34);
        debugger.state.write_ram(0x0201, 0x12);
        debugger.state.write_ram(0x0202, 0x78);
        debugger.state.write_ram(0x0203, 0x56);
        
        let info = debugger.get_sample_info(0).unwrap();
        assert_eq!(info.start_address, 0x1234);
        assert_eq!(info.loop_address, 0x5678);
    }

    #[test]
    fn test_channel_frequency() {
        let mut state = Spc700State::new();
        
        // Pitch = 0x1000 = 4096
        // Expected frequency = 4096 * 32000 / 4096 = 32000 Hz
        state.write_dsp(ChannelRegister::PitchLow.address(0), 0x00);
        state.write_dsp(ChannelRegister::PitchHigh.address(0), 0x10);
        
        let ch = state.channel_state(0);
        assert!((ch.frequency_hz() - 32000.0).abs() < 0.1);
    }
}
