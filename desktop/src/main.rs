use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use chip8_core::chip8::Chip8;
mod gui;
mod input;

const CYCLES_PER_SECOND: u16 = 700;
const TIMER_HZ: u8 = 60;
const WINDOW_SCALE: usize = 18;


pub fn main() {

    // Chip-8
    let mut chip8 = Chip8::new(); 
    let rom = std::fs::read("roms/test_opcode.ch8").expect("failed to read ROM");
    chip8.load_rom(rom.as_slice()).expect("failed to load ROM");

    // Set up GUI and input
    let sdl_context = sdl2::init().unwrap();
    let mut renderer = gui::Renderer::new(&sdl_context, WINDOW_SCALE, gui::Mode::Standard);
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // Game loop
        if let Err(e) = chip8.step() {
            eprintln!("emulation error: {e}");
            break 'running;
        }
        // Renderer
        renderer.render(chip8.get_display());
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
