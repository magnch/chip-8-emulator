extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::Duration;
use std::thread;

use chip8_core::chip8::Chip8;

const CYCLES_PER_SECOND: u16 = 700;
const TIMER_HZ: u8 = 60;

pub fn main() {

    let mut chip8 = Chip8::new(); 

    let rom = std::fs::read("roms/ibm_logo.ch8").expect("failed to read ROM");
    chip8.load_rom(rom.as_slice());

    loop {
        chip8.step();
        print_display(&chip8);
        // Update at 1 Hz
        std::thread::sleep(Duration::from_sec(1));
    }

    /* let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    } */
}

fn print_display(chip8: &Chip8) {
    for row in chip8.get_display_content() {
        for pixel in row {
            if *pixel {
                print!("*");
            } else {
                print!(" ");
            }
        }
        print!("\r\n");
    }
}