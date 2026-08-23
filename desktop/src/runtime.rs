use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError},
    thread,
    time::{Duration, Instant},
};

use crate::emulator::Emulator;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const NUM_KEYS: usize = 16;

pub enum EmuCommand {
    KeyDown(usize),
    KeyUp(usize),
    LoadRom(Vec<u8>),
    Pause(bool),
}

#[derive(Clone, Debug)]
pub struct EmuSnapshot {
    pub display_buffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    pub display_dirty: bool,
    pub beeping: bool,
    pub error: Option<String>,
}

pub struct EmulatorRuntime {
    pub command_tx: Sender<EmuCommand>,
    pub snapshot_rx: Receiver<EmuSnapshot>,
}

pub fn spawn_emulator_runtime(cpu_hz: u32) -> EmulatorRuntime {
    let (command_tx, command_rx) = mpsc::channel::<EmuCommand>();
    let (snapshot_tx, snapshot_rx) = mpsc::sync_channel::<EmuSnapshot>(1);

    thread::spawn(move || {
        emulator_worker(cpu_hz, command_rx, snapshot_tx);
    });

    EmulatorRuntime {
        command_tx,
        snapshot_rx,
    }
}

fn emulator_worker(
    cpu_hz: u32,
    command_rx: Receiver<EmuCommand>,
    snapshot_tx: SyncSender<EmuSnapshot>,
) {
    let mut emulator = Emulator::new(cpu_hz);
    let mut paused = false;
    let mut last_update = Instant::now();
    let mut last_error: Option<String> = None;

    'run: loop {
        // Drain pending commands
        'commands: loop {
            match command_rx.try_recv() {
                Ok(EmuCommand::KeyDown(key)) => {
                    if let Err(err) = emulator.key_down(key) {
                        last_error = Some(err.to_string());
                    }
                }
                Ok(EmuCommand::KeyUp(key)) => {
                    if let Err(err) = emulator.key_up(key) {
                        last_error = Some(err.to_string());
                    }
                }
                Ok(EmuCommand::LoadRom(bytes)) => {
                    match emulator.load_rom(&bytes) {
                        Ok(()) => {
                            last_error = None;
                            paused = false;
                        }
                        Err(err) => {
                            last_error = Some(err.to_string());
                        }
                    }
                }
                Ok(EmuCommand::Pause(value)) => {
                    paused = value;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // Update emulator
        let now = Instant::now();
        let elapsed = (now - last_update).min(Duration::from_millis(250));
        last_update = now;

        if !paused {
            if let Err(err) = emulator.update(elapsed) {
                last_error = Some(err.to_string());
                paused = true;
            }
        }

        // Send snapshot to GUI handler
        let snapshot = EmuSnapshot {
            display_buffer: *emulator.display().get_content(),
            display_dirty: emulator.display_take_dirty(),
            beeping: emulator.is_beeping(),
            error: last_error.clone(),
        };
        match snapshot_tx.try_send(snapshot) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                // GUI has not consumed previous frame yet; drop this snapshot.
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
        // Avoid busy spin
        thread::sleep(Duration::from_millis(1));
    }
}