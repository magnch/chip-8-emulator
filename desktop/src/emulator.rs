//! Timing wrapper around [`Chip8`] that decouples CPU and timer frequency
//! from the caller's own update rate.
//!
//! [`Emulator::update`] accumulates elapsed wall-clock time and runs however
//! many CPU steps and timer ticks are due, so callers (e.g. an egui frame
//! loop, or a dedicated emulator thread) don't need to run at CHIP-8's CPU
//! frequency themselves.

use std::time::Duration;

use chip8_core::{Chip8, Chip8Error, Config, CpuState, Display, Memory};

/// A [`Chip8`] instance paired with independent CPU and timer clocks.
pub struct Emulator {
    chip8: Chip8,
    cpu_step: Duration,
    timer_step: Duration,
    cpu_accumulator: Duration,
    timer_accumulator: Duration,
}

impl Default for Emulator {
    /// Create an emulator running at [`Emulator::DEFAULT_CPU_HZ`].
    fn default() -> Self {
        Emulator::new(Self::DEFAULT_CPU_HZ)
    }
}

impl Emulator {
    const TIMER_HZ: u32 = 60;
    const DEFAULT_CPU_HZ: u32 = 700;

    /// Create a reset emulator that executes `cpu_hz` instructions per second.
    pub fn new(cpu_hz: u32) -> Self {
        Emulator {
            chip8: Chip8::default(),
            cpu_step: Duration::from_secs_f64(1.0 / cpu_hz as f64),
            timer_step: Duration::from_secs_f64(1.0 / Self::TIMER_HZ as f64),
            cpu_accumulator: Duration::ZERO,
            timer_accumulator: Duration::ZERO,
        }
    }

    /// Advance the emulator by `elapsed` wall-clock time, running every CPU
    /// step and timer tick that is due. Stops at the first instruction error.
    pub fn update(&mut self, elapsed: Duration) -> Result<(), Chip8Error> {
        // Update accumulators
        self.cpu_accumulator += elapsed;
        self.timer_accumulator += elapsed;
        // Run every elapsed CPU cycle, even when the GUI frame rate is lower.
        while self.cpu_accumulator >= self.cpu_step {
            self.chip8.step()?;
            self.cpu_accumulator -= self.cpu_step;
        }

        while self.timer_accumulator >= self.timer_step {
            self.chip8.tick_timers();
            self.timer_accumulator -= self.timer_step;
        }
        Ok(())
    }

    /// Clear all CPU, memory, and display state back to a fresh boot.
    /// Does not reload a ROM — follow with [`Emulator::load_rom`].
    pub fn reset(&mut self) {
        self.chip8.reset();
    }

    /// Mark a CHIP-8 key as pressed.
    pub fn key_down(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.chip8.key_down(key)
    }

    /// Perform a single CPU step
    pub fn step(&mut self) {
        let _ = self.chip8.step();
    }

    /// Mark a CHIP-8 key as released.
    pub fn key_up(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.chip8.key_up(key)
    }

    /// Return whether the sound timer is active.
    pub fn is_beeping(&self) -> bool {
        self.chip8.is_beeping()
    }

    /// Borrow the current display object.
    pub fn display(&self) -> &Display {
        self.chip8.get_display()
    }

    /// Take the display's dirty flag, returning whether it has changed since
    /// the last call.
    pub fn display_take_dirty(&mut self) -> bool {
        self.chip8.display_take_dirty()
    }

    /// Load a ROM and reset the program counter to its start address.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        self.chip8.load_rom(rom)
    }

    /// Replace the interpreter's compatibility settings, taking effect on the
    /// next instruction.
    pub fn set_config(&mut self, config: Config) {
        self.chip8.config = config;
    }
}

/// Debugger methods
impl Emulator {
    pub fn get_state(&self) -> CpuState {
        self.chip8.get_state()
    }

    pub fn get_memory_content(&self) -> &[u8; Memory::MEMORY_SIZE] {
        self.chip8.get_memory_content()
    }
}