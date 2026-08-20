extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use chip8_core::display::Display;

pub enum Mode {
    
    Debugger,
}

pub struct Renderer {
    canvas: Canvas<Window>,
    scale: usize,
    mode: Mode,
}

impl Renderer {
    pub fn new(sdl_context: &sdl2::Sdl, scale: usize, mode: Mode) -> Self {
        // SDL2 initialization
        let width = (Display::WIDTH * scale) as u32;
        let height = (Display::HEIGHT * scale) as u32;

        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("chip-8 emulator", width, height)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        // Clear window
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Self {
            canvas,
            scale,
            mode,
        }
    }

    pub fn render(&mut self, display: &Display) {
        // Clear screen and set draw color to white
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        // Draw each pixel
        for y in 0..Display::HEIGHT {
            for x in 0..Display::WIDTH {
                if display.get_content()[y][x] {
                    self.draw_pixel(x, y);
                }
            }
        }
        // Update window
        self.canvas.present();
    }

    fn draw_pixel(&mut self, x: usize, y: usize) {
        let pixel = Rect::new(
            (x * self.scale) as i32,
            (y * self.scale) as i32,
            self.scale as u32,
            self.scale as u32,
        );
        self.canvas.fill_rect(pixel).unwrap();
    }
}
      );
        self.canvas.fill_rect(pixel).unwrap();
    }
}       self.canvas.fill_rect(pixel).unwrap();
    }
}