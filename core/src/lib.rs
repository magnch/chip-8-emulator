//! CHIP-8 emulator core.
//!
//! `chip8-core` implements the CHIP-8 virtual machine without any platform
//! dependencies for graphics, audio, or keyboard input. A frontend owns the
//! event loop and uses [`chip8::Chip8`] to load a ROM, execute instructions,
//! update timers, read the display, and forward key events.
//!
//! ## Execution model
//!
//! [`chip8::Chip8::step`] executes one instruction. Timer updates are
//! separate and are performed with [`chip8::Chip8::tick_timers`]. This lets a
//! frontend schedule CPU execution and the 60 Hz timers independently.
//!
//! ## Example
//!
//! ```
//! use chip8_core::chip8::Chip8;
//!
//! # fn run(rom: &[u8]) -> Result<(), chip8_core::error::Chip8Error> {
//! let mut emulator = Chip8::new();
//! emulator.load_rom(rom)?;
//! emulator.step()?;
//! emulator.tick_timers();
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`chip8`] contains the emulator and CPU execution loop.
//! - [`config`] contains compatibility settings used by the interpreter.
//! - [`display`] provides the 64 x 32 display buffer.
//! - [`error`] defines errors returned by the core.
//! - [`memory`] provides the CHIP-8 memory abstraction.

pub mod chip8;
pub mod config;
pub mod display;
pub mod error;
pub mod memory;

mod font;
mod keypad;
mod opcode;
mod utils;