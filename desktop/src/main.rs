use eframe::egui;

mod audio;
mod emulator;
mod gui;
mod runtime;

// Application constants
const CPU_HZ: u32 = 700;
const WINDOW_SCALE: usize = 18;

pub fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([64.0 * WINDOW_SCALE as f32, 32.0 * WINDOW_SCALE as f32]),
        ..Default::default()
    };

    let runtime = runtime::spawn_emulator_runtime(CPU_HZ);

    eframe::run_native(
        "CHIP-8 Emulator",
        options,
        Box::new(move |_creation_context| Ok(Box::new(gui::Chip8App::new(runtime, None)))),
    )
    .expect("failed to start CHIP-8 window");
}
