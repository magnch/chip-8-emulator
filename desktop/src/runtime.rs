//! Runs the emulator on a dedicated background thread and exposes it to the
//! GUI thread through two channels.
//!
//! The emulator must be stepped at its own CPU/timer rate regardless of how
//! often egui repaints, so [`spawn_emulator_runtime`] hands it its own
//! thread: [`EmuCommand`]s (key presses, loading a ROM, pausing, ...) flow
//! in, and an [`EmuSnapshot`] of the latest display/audio/error state flows
//! back out for the GUI to render each frame.

use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError},
    thread,
    time::{Duration, Instant},
};

use chip8_core::{Config, CpuState};

use crate::emulator::Emulator;

/// Display width in pixels, mirrors [`chip8_core::Display::WIDTH`].
pub const DISPLAY_WIDTH: usize = 64;
/// Display height in pixels, mirrors [`chip8_core::Display::HEIGHT`].
pub const DISPLAY_HEIGHT: usize = 32;
/// Number of keys on the CHIP-8 keypad.
pub const NUM_KEYS: usize = 16;

/// A request sent from the GUI thread to the emulator thread.
pub enum EmuCommand {
    /// Mark a CHIP-8 key (`0..16`) as pressed.
    KeyDown(usize),
    /// Mark a CHIP-8 key (`0..16`) as released.
    KeyUp(usize),
    /// Load a ROM image and start executing it from its entry point.
    LoadRom(Vec<u8>),
    /// Pause (`true`) or resume (`false`) CPU and timer execution.
    Pause(bool),
    /// Clear all CPU, memory, and display state back to a fresh boot.
    Reset(),
    /// Replace the interpreter's compatibility settings.
    SetConfig(Config),
    /// Execute a single CPU step, regardless of the pause state. Used by
    /// the debugger's Step button.
    StepOnce(),
}

/// A snapshot of emulator state produced once per worker loop iteration,
/// for the GUI thread to render without touching the emulator directly.
#[derive(Clone, Debug)]
pub struct EmuSnapshot {
    /// The current display framebuffer.
    pub display_buffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    /// Whether the display has changed since the previous snapshot.
    pub display_dirty: bool,
    /// Whether the sound timer is currently active.
    pub beeping: bool,
    /// The most recent emulator error, if any, as a display string.
    pub error: Option<String>,
    /// The current CPU state, for the debugger panel.
    pub cpu: CpuState,
    /// The full memory space, for the debugger's instruction disassembly.
    pub memory: [u8; 4096],
}


/// Handle to a running emulator thread: send [`EmuCommand`]s in, receive
/// [`EmuSnapshot`]s out.
pub struct EmulatorRuntime {
    /// Sends commands to the emulator thread.
    pub command_tx: Sender<EmuCommand>,
    /// Receives the latest emulator snapshot. Capacity 1: the emulator
    /// thread drops new snapshots if the GUI hasn't consumed the previous
    /// one yet, so this always yields the most recent state, not a queue.
    pub snapshot_rx: Receiver<EmuSnapshot>,
}

/// Spawn the emulator on its own thread, running its CPU at `cpu_hz`.
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

/// The emulator thread's main loop: drain pending commands, advance the
/// emulator by however much time has passed, then publish a snapshot.
fn emulator_worker(
    cpu_hz: u32,
    command_rx: Receiver<EmuCommand>,
    snapshot_tx: SyncSender<EmuSnapshot>,
) {
    let mut emulator = Emulator::new(cpu_hz);
    let mut paused = false;
    let mut last_update = Instant::now();
    let mut last_error: Option<String> = None;

    loop {
        // Drain pending commands
        loop {
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
                Ok(EmuCommand::LoadRom(bytes)) => match emulator.load_rom(&bytes) {
                    Ok(()) => {
                        last_error = None;
                        paused = false;
                    }
                    Err(err) => {
                        last_error = Some(err.to_string());
                    }
                },
                Ok(EmuCommand::Pause(value)) => {
                    paused = value;
                }
                Ok(EmuCommand::Reset()) => {
                    emulator.reset();
                }
                Ok(EmuCommand::SetConfig(config)) => {
                    emulator.set_config(config);
                }
                Ok(EmuCommand::StepOnce()) => {
                    emulator.step();
                }

                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // Update emulator
        let now = Instant::now();
        let elapsed = (now - last_update).min(Duration::from_millis(250));
        last_update = now;

        if !paused && let Err(err) = emulator.update(elapsed) {
            last_error = Some(err.to_string());
            paused = true;
        }

        // Send snapshot to GUI handler
        let snapshot = EmuSnapshot {
            display_buffer: *emulator.display().get_content(),
            display_dirty: emulator.display_take_dirty(),
            beeping: emulator.is_beeping(),
            error: last_error.clone(),
            cpu: emulator.get_state(),
            memory: *emulator.get_memory_content(),
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
