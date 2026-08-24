//! CHIP-8 emulator core.
//!
//! `chip8-core` implements the CHIP-8 virtual machine without any platform
//! dependencies for graphics, audio, or keyboard input. A frontend owns the
//! event loop and uses [`Chip8`] to load a ROM, execute instructions, update
//! timers, read the display, and forward key events.
//!
//! ## Execution model
//!
//! [`Chip8::step`] executes one instruction. Timer updates are separate and
//! are performed with [`Chip8::tick_timers`]. This lets a frontend schedule
//! CPU execution and the 60 Hz timers independently.
//!
//! ## Example
//!
//! ```
//! use chip8_core::Chip8;
//!
//! # fn run(rom: &[u8]) -> Result<(), chip8_core::Chip8Error> {
//! let mut emulator = Chip8::new();
//! emulator.load_rom(rom)?;
//! emulator.step()?;
//! emulator.tick_timers();
//! # Ok(())
//! # }
//! ```
//!
//! ## Public API
//!
//! - [`Chip8`] is the interpreter: load a ROM, step, tick timers, read the display, forward key events.
//! - [`Config`] holds the compatibility toggles applied through [`Chip8::config`].
//! - [`Display`] is the 64 x 32 framebuffer returned by [`Chip8::get_display`].
//! - [`Chip8Error`] is returned when an operation cannot complete.
//!
//! ## `debug-tools` feature
//!
//! Enabling the `debug-tools` Cargo feature additionally exposes
//! [`CpuState`], [`Memory`], [`Instruction`], and [`decode`], along with
//! [`Chip8::get_state`], [`Chip8::get_memory_content`],
//! [`Chip8::set_register`], and [`Chip8::set_pc`] — the read/write
//! introspection surface a debugger frontend needs, kept out of the default
//! build otherwise.

mod chip8;
mod config;
mod display;
mod error;
mod font;
mod keypad;
mod memory;
mod opcode;
mod utils;

pub use chip8::Chip8;
#[cfg(feature = "debug-tools")]
pub use chip8::CpuState;
#[cfg(feature = "debug-tools")]
pub use memory::Memory;
#[cfg(feature = "debug-tools")]
pub use opcode::{Instruction, decode};

pub use config::Config;
pub use display::Display;
pub use error::Chip8Error;
