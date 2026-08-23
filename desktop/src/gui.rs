//! The egui [`eframe::App`] that renders the CHIP-8 display, menu bar, and
//! error banner, and forwards keyboard input to the emulator thread.

use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use chip8_core::{Config, CpuState, Display};
use eframe::egui;
use egui::{Color32, Rect, Vec2};

use crate::audio::AudioPlayer;
use crate::runtime::{self, EmuCommand, EmuSnapshot, EmulatorRuntime};

/// The CHIP-8 desktop application's egui state.
///
/// Owns the emulator thread handle and the last [`EmuSnapshot`] it sent;
/// per-frame work is to forward input, drain self.snapshots, and paint.
pub struct Chip8App {
    runtime: EmulatorRuntime,
    snapshot: EmuSnapshot,
    /// The compatibility settings currently applied, as last pushed to the
    /// emulator thread via [`EmuCommand::SetConfig`]. The GUI is the source
    /// of truth for this value; the emulator thread never reports it back.
    config: Config,
    audio_player: AudioPlayer,
    previous_keys: [bool; 16],
    error: Option<String>,
    /// The ROM currently loaded, kept so Settings > Reset can reload it.
    loaded_rom: Option<Vec<u8>>,
    paused: bool,
    show_debugger: bool,
}

impl Chip8App {
    /// Create the app and, if `initial_rom` is given, load it immediately.
    pub fn new(runtime: EmulatorRuntime, initial_rom: Option<Vec<u8>>) -> Self {
        let initial_snapshot = EmuSnapshot {
            display_buffer: [[false; runtime::DISPLAY_WIDTH]; runtime::DISPLAY_HEIGHT],
            display_dirty: false,
            beeping: false,
            error: None,
            cpu: CpuState::default(),
            memory: [0; 4096],
        };

        if let Some(rom) = &initial_rom {
            let _ = runtime.command_tx.send(EmuCommand::LoadRom(rom.clone()));
        }

        Self {
            runtime,
            snapshot: initial_snapshot,
            config: Config::default(),
            audio_player: AudioPlayer::default(),
            previous_keys: [false; runtime::NUM_KEYS],
            error: None,
            loaded_rom: initial_rom,
            paused: false,
            show_debugger: false,
        }
    }

    /// Draw the File / Settings menu bar: loading ROMs, exiting, pausing,
    /// resetting, and toggling compatibility [`Config`] flags live.
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
                    ui.menu_button("Configuration", |ui| {
                        if ui
                            .checkbox(&mut self.config.adi_flags_overflow, "Adi flags overflow")
                            .changed()
                        {
                            let _ = self
                                .runtime
                                .command_tx
                                .send(EmuCommand::SetConfig(self.config));
                        }
                        if ui
                            .checkbox(&mut self.config.jmi_uses_vx, "Jmi uses Vx register")
                            .changed()
                        {
                            let _ = self
                                .runtime
                                .command_tx
                                .send(EmuCommand::SetConfig(self.config));
                        }
                        if ui
                            .checkbox(&mut self.config.shift_uses_vy, "Shift uses Vy register")
                            .changed()
                        {
                            let _ = self
                                .runtime
                                .command_tx
                                .send(EmuCommand::SetConfig(self.config));
                        }
                        if ui
                            .checkbox(
                                &mut self.config.sprites_wrap_at_edge,
                                "Sprites wrap at edge of display",
                            )
                            .changed()
                        {
                            let _ = self
                                .runtime
                                .command_tx
                                .send(EmuCommand::SetConfig(self.config));
                        }
                        if ui
                            .checkbox(
                                &mut self.config.str_ldr_increments_index,
                                "Store and load increment index",
                            )
                            .changed()
                        {
                            let _ = self
                                .runtime
                                .command_tx
                                .send(EmuCommand::SetConfig(self.config));
                        }
                    });
                    if ui.checkbox(&mut self.show_debugger, "Show Debugger").changed() {}
                });
            });
        });
    }

    fn draw_debugger(&mut self, ctx: &egui::Context) {
        if !self.show_debugger {
            return;
        }
        self.draw_control_panel(ctx);
        self.draw_cpu_state_panel(ctx, &self.snapshot);
        self.draw_instructions_panel(ctx, &self.snapshot);
    }

    fn draw_cpu_state_panel(&self, ctx: &egui::Context, snapshot: &EmuSnapshot) {
        let window_width = ctx.screen_rect().width();
        let panel_width = window_width * 0.18; // 18% of the window

        egui::SidePanel::left("cpu_state_panel")
            .exact_width(panel_width)
            .show(ctx, |ui| {
                ui.heading("Registers");
                egui::Grid::new("registers_grid").show(ui, |ui| {
                    for (i, reg) in self.snapshot.cpu.registers.iter().enumerate() {
                        ui.label(format!("V{i:X}"));
                        ui.label(format!("{reg:#04X}"));
                        if i % 2 == 1 { ui.end_row(); }
                    }
                });

                ui.separator();
                ui.label(format!("PC: {:#05X}", snapshot.cpu.pc));
                ui.label(format!("I:  {:#05X}", snapshot.cpu.index));
                ui.label(format!("SP: {}", snapshot.cpu.sp));
                ui.label(format!("DT: {}", snapshot.cpu.delay_timer));
                ui.label(format!("ST: {}", snapshot.cpu.sound_timer));

                ui.separator();
                ui.heading("Stack");
                for (i, addr) in snapshot.cpu.stack.iter().take(snapshot.cpu.sp as usize).enumerate() {
                    ui.label(format!("{i}: {addr:#05X}"));
                }
            });
    }

    fn draw_instructions_panel(&self, ctx: &egui::Context, snapshot: &EmuSnapshot) {
        let window_width = ctx.screen_rect().width();
        let panel_width = window_width * 0.16; // 16% of the window

        egui::SidePanel::right("instructions_panel")
            .exact_width(panel_width)
            .show(ctx, |ui| {
                ui.heading("Instructions");

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let pc = self.snapshot.cpu.pc;
                    let start = pc.saturating_sub(10) & !1; // stay word-aligned, show some context above
                    let mut addr = start;

                    while addr < start + 40 && (addr as usize) < snapshot.memory.len() - 1 {
                        let word = u16::from_be_bytes([
                            snapshot.memory[addr as usize],
                            snapshot.memory[addr as usize + 1],
                        ]);

                        let instr = chip8_core::decode(word);
                        let text = if matches!(instr, chip8_core::Instruction::Unknown(_word)) {
                            format!("??? {word:#06X}")
                        } else {
                            instr.to_string()
                        };

                        let line = format!("{addr:#05X}: {text}");

                        if addr == pc {
                            ui.colored_label(egui::Color32::YELLOW, format!("▶ {line}"));
                        } else {
                            ui.monospace(line);
                        }

                        addr += 2;
                    }
                });
            });
    }

    fn draw_control_panel(&mut self, ctx: &egui::Context) {
        let window_height = ctx.screen_rect().height();
        let panel_height = window_height * 0.20;
        let button_size = egui::Vec2::new(100.0, 40.0);

        egui::TopBottomPanel::bottom("control_panel")
            .exact_height(panel_height)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(panel_height * 0.5 - 20.0); // rough vertical centering
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 2.0 - (button_size.x * 3.0 + 16.0) / 2.0);

                        let pause_label = if self.paused { "Resume" } else { "Pause" };
                        if ui.add(egui::Button::new(pause_label).min_size(button_size)).clicked() {
                            self.paused = !self.paused;
                            let _ = self.runtime.command_tx.send(EmuCommand::Pause(self.paused));
                        }

                        if ui
                            .add_enabled(self.paused, egui::Button::new("Step").min_size(button_size))
                            .clicked()
                        {
                            let _ = self.runtime.command_tx.send(EmuCommand::StepOnce());
                        }

                        if ui
                            .add_enabled(
                                self.loaded_rom.is_some(),
                                egui::Button::new("Reset").min_size(button_size),
                            )
                            .clicked()
                        {
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

    /// Paint the current display buffer, scaled to fill the available space
    /// while preserving the CHIP-8's 64x32 aspect ratio.
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

    /// Compare this frame's key states against the last frame's and send a
    /// [`EmuCommand::KeyDown`]/[`EmuCommand::KeyUp`] for each key that changed.
    fn send_input_edges(&mut self, current_keys: [bool; 16]) {
        for (key, _) in current_keys.iter().enumerate() {
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

    /// Replace [`Chip8App::snapshot`] with the most recent one available,
    /// discarding any older, already-superseded self.snapshots.
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
    /// Run one egui frame: forward input, pull in the latest emulator
    /// state, and paint the menu bar and display.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.send_input_edges(keyboard_state(ctx));
        self.drain_latest_snapshot();

        self.audio_player.set_playing(self.snapshot.beeping);

        if let Some(err) = &self.snapshot.error {
            self.error = Some(err.clone());
        }

        self.draw_menu_bar(ctx);
        self.draw_debugger(ctx);

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

/// Read the current down/up state of all 16 CHIP-8 keys from the keyboard.
///
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
