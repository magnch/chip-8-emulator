use std::time::{Duration, Instant};

use chip8_core::chip8::Chip8;
use chip8_core::error::Chip8Error;

pub struct Emulator {
    chip8: Chip8,
    cpu_step: Duration,
    timer_step: Duration,
    cpu_accumulator: Duration,
    timer_accumulator: Duration,
}

impl Default for Emulator {
    fn default() -> Self {
        Emulator::new(Self::DEFAULT_CPU_CYCLES_PER_SECOND)
    }
}

impl Emulator {
    const TIMER_HZ: u32 = 60;
    const DEFAULT_CPU_CYCLES_PER_SECOND: u32 = 700;

    pub fn new(cpu_cycles_per_second: u32) -> Self {
        Emulator {
            chip8: Chip8::default(),
            cpu_step: Duration::from_secs_f64(1.0 / cpu_cycles_per_second as f64),
            timer_step: Duration::from_secs_f64(1.0 / Self::TIMER_HZ as f64),
            cpu_accumulator: Duration::ZERO,
            timer_accumulator: Duration::ZERO,
        }
    }

    pub fn update(&mut self, elapsed: Duration) -> Result<(), Chip8Error> {
        // Update accumulators
        self.cpu_accumulator += elapsed;
        self.timer_accumulator += elapsed;
        // CPU cycle
        if self.cpu_accumulator >= self.cpu_step {
            self.chip8.step()?;
            self.cpu_accumulator -= self.cpu_step;
        }
        // Timer cycle
        if self.timer_accumulator >= self.timer_step {
            self.chip8.tick_timers();
            self.timer_accumulator -= self.timer_step;
        }
        Ok(())
    }

    pub fn key_down(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.chip8.key_down(key)
    }

    pub fn key_up(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.chip8.key_up(key)
    }

    pub fn display(&self) -> &Display {
        self.chip8.get_display()
    }
}