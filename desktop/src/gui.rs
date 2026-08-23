use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use chip8_core::display::Display;
use eframe::egui;
use egui::{Color32, Rect, Vec2};

use crate::audio::AudioPlayer;
use crate::runtime::{self, EmuCommand, EmuSnapshot, EmulatorRuntime};

pub struct Chip8App {
    runtime: EmulatorRuntime,
    snapshot: EmuSnapshot,
    audio_player: AudioPlayer,
    previous_keys: [bool; 16],
    error: Option<String>,
    loaded_rom: Option<Vec<u8>>,
    paused: bool,
}

impl Chip8App {
    pub fn new(runtime: EmulatorRuntime, initial_rom: Option<Vec<u8>>) -> Self {
        let initial_snapshot = EmuSnapshot {
            display_buffer: [[false; runtime::DISPLAY_WIDTH]; runtime::DISPLAY_HEIGHT],
            display_dirty: false,
            beeping: false,
            error: None,
        };

        if let Some(rom) = &initial_rom {
            let _ = runtime.command_tx.send(EmuCommand::LoadRom(rom.clone()));
        }

        Self {
            runtime,
            snapshot: initial_snapshot,
            audio_player: AudioPlayer::default(),
            previous_keys: [false; runtime::NUM_KEYS],
            error: None,
            loaded_rom: initial_rom,
            paused: false,
        }
    }

    fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM…").clicked() {
                        ui.close_menu();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("CHIP-8 ROM", &["ch8", "c8", "bin"])
                            .pick_file()
                        {
                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    let _ = self.runtime.command_tx.send(EmuCommand::Reset());
                                    let _ = self
                                        .runtime
                                        .command_tx
                                        .send(EmuCommand::LoadRom(bytes.clone()));
                                    self.loaded_rom = Some(bytes);
                                    self.paused = false;
                                    self.error = None;
                                }
                                Err(err) => self.error = Some(err.to_string()),
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.close_menu();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.checkbox(&mut self.paused, "Paused").changed() {
                        let _ = self.runtime.command_tx.send(EmuCommand::Pause(self.paused));
                    }
                    if ui
                        .add_enabled(self.loaded_rom.is_some(), egui::Button::new("Reset"))
                        .clicked()
                    {
                        ui.close_menu();
                        if let Some(rom) = self.loaded_rom.clone() {
                            let _ = self.runtime.command_tx.send(EmuCommand::Reset());
                            let _ = self.runtime.command_tx.send(EmuCommand::LoadRom(rom));
                            self.paused = false;
                        }
                    }
                });
            });
        });
    }

    fn draw_display(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let scale = (available.x / runtime::DISPLAY_WIDTH as f32)
            .min(available.y / runtime::DISPLAY_HEIGHT as f32);

        let display_size = Vec2::new(
            Display::WIDTH as f32 * scale,
            Display::HEIGHT as f32 * scale,
        );

        let (response, painter) = ui.allocate_painter(display_size, egui::Sense::hover());
        let origin = response.rect.min;

        painter.rect_filled(response.rect, 0.0, Color32::BLACK);

        for y in 0..Display::HEIGHT {
            for x in 0..Display::WIDTH {
                if self.snapshot.display_buffer[y][x] {
                    let top_left = origin + Vec2::new(x as f32 * scale, y as f32 * scale);
                    let pixel = Rect::from_min_size(top_left, Vec2::splat(scale));
                    painter.rect_filled(pixel, 0.0, Color32::WHITE);
                }
            }
        }
    }

    fn send_input_edges(&mut self, current_keys: [bool; 16]) {
        for key in 0..16 {
            let was_down = self.previous_keys[key];
            let is_down = current_keys[key];

            if !was_down && is_down {
                let _ = self.runtime.command_tx.send(EmuCommand::KeyDown(key));
            } else if was_down && !is_down {
                let _ = self.runtime.command_tx.send(EmuCommand::KeyUp(key));
            }
        }

        self.previous_keys = current_keys;
    }

    fn drain_latest_snapshot(&mut self) {
        loop {
            match self.runtime.snapshot_rx.try_recv() {
                Ok(snapshot) => {
                    self.snapshot = snapshot;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.error = Some("emulator thread disconnected".to_string());
                    break;
                }
            }
        }
    }
}

impl eframe::App for Chip8App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.send_input_edges(keyboard_state(ctx));
        self.drain_latest_snapshot();

        self.audio_player.set_playing(self.snapshot.beeping);

        if let Some(err) = &self.snapshot.error {
            self.error = Some(err.clone());
        }

        self.draw_menu_bar(ctx);

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
