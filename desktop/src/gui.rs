use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Rect, Vec2};

use chip8_core::display::Display;

use crate::{audio::AudioPlayer, emulator::Emulator};

pub struct Chip8App {
    pub emulator: Emulator,
    pub audio_player: AudioPlayer,
    pub error: Option<String>,
    last_update: Instant,
}

impl Chip8App {
    pub fn new(cpu_cycles_per_second: u32) -> Self {
        Self {
            emulator: Emulator::new(cpu_cycles_per_second),
            audio_player: AudioPlayer::default(),
            error: None,
            last_update: Instant::now(),
        }
    }

    fn draw_display(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let scale = (available.x / Display::WIDTH as f32).min(available.y / Display::HEIGHT as f32);

        let display_size = Vec2::new(
            Display::WIDTH as f32 * scale,
            Display::HEIGHT as f32 * scale,
        );

        let (response, painter) = ui.allocate_painter(display_size, egui::Sense::hover());
        let origin = response.rect.min;

        painter.rect_filled(response.rect, 0.0, Color32::BLACK);

        for y in 0..Display::HEIGHT {
            for x in 0..Display::WIDTH {
                if self.emulator.display().get_content()[y][x] {
                    let top_left = origin + Vec2::new(x as f32 * scale, y as f32 * scale);
                    let pixel = Rect::from_min_size(top_left, Vec2::splat(scale));
                    painter.rect_filled(pixel, 0.0, Color32::WHITE);
                }
            }
        }
    }
}

impl eframe::App for Chip8App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Timing
        let now = Instant::now();
        let elapsed = (now - self.last_update).min(Duration::from_millis(100));
        self.last_update = now;

        // Run emulator
        if let Err(error) = self.emulator.update(elapsed) {
            self.error = Some(error.to_string());
        }

        // Handle input
        for (key, pressed) in keyboard_state(ctx).into_iter().enumerate() {
            let result = if pressed {
                self.emulator.key_down(key)
            } else {
                self.emulator.key_up(key)
            };
            if let Err(error) = result {
                self.error = Some(error.to_string());
            }
        }

        // Handle sound
        self.audio_player.set_playing(self.emulator.is_beeping());

        // Handle GUI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                self.draw_display(ui);
            });

            if let Some(error) = &self.error {
                ui.colored_label(Color32::RED, error);
            }
        });

        ctx.request_repaint_after(Duration::from_millis(8));
    }
}

/// Layout (physical position -> CHIP-8 key):
///   1 2 3 4        1 2 3 C
///   Q W E R   ->   4 5 6 D
///   A S D F        7 8 9 E
///   Z X C V        A 0 B F
fn keyboard_state(ctx: &egui::Context) -> [bool; 16] {
    let mut keys = [false; 16];

    ctx.input(|input| {
        let mapping = [
            (egui::Key::Num1, 0x1),
            (egui::Key::Num2, 0x2),
            (egui::Key::Num3, 0x3),
            (egui::Key::Num4, 0xC),
            (egui::Key::Q, 0x4),
            (egui::Key::W, 0x5),
            (egui::Key::E, 0x6),
            (egui::Key::R, 0xD),
            (egui::Key::A, 0x7),
            (egui::Key::S, 0x8),
            (egui::Key::D, 0x9),
            (egui::Key::F, 0xE),
            (egui::Key::Z, 0xA),
            (egui::Key::X, 0x0),
            (egui::Key::C, 0xB),
            (egui::Key::V, 0xF),
        ];

        for (key, chip8_key) in mapping {
            keys[chip8_key] = input.key_down(key);
        }
    });

    keys
}
