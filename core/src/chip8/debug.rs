//! Introspection and direct-mutation methods for a debugger frontend.
//!
//! Everything here is gated behind the `debug-tools` Cargo feature: it lets
//! a frontend read CPU/memory state and edit registers or the program
//! counter directly, which a plain interpreter consumer has no need for.

use super::Chip8;
use crate::memory::Memory;

/// Snapshot of the CPU state, for a debugger to display.
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    /// The 16 general-purpose registers `V0`..`VF`.
    pub registers: [u8; Chip8::NUM_REGS],
    /// The program counter.
    pub pc: usize,
    /// The index register (`I`).
    pub index: usize,
    /// The number of return addresses currently on the call stack.
    pub sp: usize,
    /// The call stack, of which only the first `sp` entries are in use.
    pub stack: [u16; Chip8::STACK_SIZE],
    /// The delay timer's current value.
    pub delay_timer: u8,
    /// The sound timer's current value.
    pub sound_timer: u8,
}

impl Default for CpuState {
    fn default() -> Self {
        CpuState {
            registers: [0; Chip8::NUM_REGS],
            pc: 0,
            index: 0,
            sp: 0,
            stack: [0; Chip8::STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl Chip8 {
    /// Copy the current CPU state for inspection.
    pub fn get_state(&self) -> CpuState {
        CpuState {
            registers: self.registers,
            pc: self.pc,
            index: self.index,
            sp: self.sp,
            stack: self.stack,
            delay_timer: self.delay_timer,
            sound_timer: self.sound_timer,
        }
    }

    /// Borrow the full 4 KiB memory space, e.g. to disassemble around the program counter.
    pub fn get_memory_content(&self) -> &[u8; Memory::MEMORY_SIZE] {
        self.ram.get_content()
    }

    /// Directly overwrite a general-purpose register (`V0`..`VF`).
    pub fn set_register(&mut self, x: usize, value: u8) {
        self.registers[x] = value;
    }

    /// Directly overwrite the program counter, e.g. to set a breakpoint-style jump.
    pub fn set_pc(&mut self, addr: usize) {
        self.pc = addr;
    }
}
