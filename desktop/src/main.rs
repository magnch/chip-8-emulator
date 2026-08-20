use std::time::{Duration, Instant};

mod audio;
mod emulator;
mod gui;
mod input;

const CYCLES_PER_SECOND: u32 = 700;
const WINDOW_SCALE: usize = 18;

pub fn main() {
    // Start emulator instance
    let mut emulator = emulator::Emulator::new(CYCLES_PER_SECOND);

    // Timing
    let mut last_frame = Instant::now();

    'running: loop {
        // Timing
        let now = Instant::now();
        let elapsed = (now - last_frame).min(Duration::from_millis(100)); // cap against stalls
        last_frame = now;
        
        // Update emulator state
        emulator.update(elapsed);
    }

}
