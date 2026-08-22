use eframe::egui;

use crate::gui::Chip8App;

mod audio;
mod emulator;
mod gui;

// Application constants
const CPU_CYCLES_PER_SECOND: u32 = 700;
const WINDOW_SCALE: usize = 18;

pub fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([64.0 * WINDOW_SCALE as f32, 32.0 * WINDOW_SCALE as f32]),
        ..Default::default()
    };

    let rom = std::fs::read("roms/tests/7-beep.ch8").expect("load ROM");

    eframe::run_native(
        "CHIP-8 Emulator",
        options,
        Box::new(|_creation_context| {
            let mut app = Chip8App::new(CPU_CYCLES_PER_SECOND);
            app.emulator
                .load_rom(&rom)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)?;
            Ok(Box::new(app))
        }),
    )
    .expect("failed to start CHIP-8 window");
}
