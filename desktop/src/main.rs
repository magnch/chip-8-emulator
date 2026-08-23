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

    // Load ROM
    let rom = std::fs::read("roms/games/snake.ch8").expect("load ROM");
    runtime
        .command_tx
        .send(runtime::EmuCommand::LoadRom((rom)))
        .expect("send  ROM to emulator thread");

    eframe::run_native(
        "CHIP-8 Emulator",
        options,
        Box::new(move |_creation_context| Ok(Box::new(gui::Chip8App::new(runtime)))),
    )
    .expect("failed to start CHIP-8 window");
}
