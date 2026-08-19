use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

use chip8_core::chip8::Chip8;
mod gui;
mod input;

const CYCLES_PER_SECOND: u16 = 700;
const TIMER_HZ: u8 = 60;
const WINDOW_SCALE: usize = 18;


pub fn main() {

    // Chip-8
    let mut chip8 = Chip8::new(); 
    let rom = std::fs::read("roms/tests/6-keypad.ch8").expect("failed to read ROM");
    chip8.load_rom(rom.as_slice()).expect("failed to load ROM");

    // Set up GUI and input
    let sdl_context = sdl2::init().unwrap();
    let mut renderer = gui::Renderer::new(&sdl_context, WINDOW_SCALE, gui::Mode::Standard);
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Set up timing
    let cpu_step_duration = Duration::from_secs_f64(1.0 / CYCLES_PER_SECOND as f64);
    let timer_step_duration = Duration::from_secs_f64(1.0 / TIMER_HZ as f64);
    let mut cpu_accumulator = Duration::ZERO;
    let mut timer_accumulator = Duration::ZERO;
    let mut last_frame = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { scancode: Some(sc), repeat:false, .. } => {
                    if let Some(chip8_key) = input::map_scancode(sc) {
                        //println!("Key down: {chip8_key:#02X}");
                        chip8.key_down(chip8_key).expect("map_scancode returns a valid CHIP-8 key");
                    }
                }
                Event::KeyUp { scancode: Some(sc), .. } => {
                    if let Some(chip8_key) = input::map_scancode(sc) {
                        chip8.key_up(chip8_key).expect("map_scancode returns a valid CHIP-8 key");
                    }
                }

                _ => {}
            }
        }
        // Timing
        let now = Instant::now();
        let elapsed = (now - last_frame).min(Duration::from_millis(100)); // cap against stalls
        last_frame = now;

        cpu_accumulator += elapsed;
        timer_accumulator += elapsed;

        // CPU loop
        if cpu_accumulator >= cpu_step_duration { 
            if let Err(e) = chip8.step() {
                eprintln!("emulation error: {e}");
                break 'running;
            }
            cpu_accumulator -= cpu_step_duration;
        }
        // Timer loop
        if timer_accumulator >= timer_step_duration {
            chip8.tick_timers();
            timer_accumulator -= timer_step_duration;
        }
        // Renderer
        renderer.render(chip8.get_display());
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 700));
    }
}