use super::Chip8;
use crate::memory::Memory;

/// Snapshot of the CPU state.
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    pub registers: [u8; Chip8::NUM_REGS],
    pub pc: usize,
    pub index: usize,
    pub sp: usize,
    pub stack: [u16; Chip8::STACK_SIZE],
    pub delay_timer: u8,
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
            sound_timer: 0
        }
    }
}

impl Chip8 {
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

    pub fn get_memory_content(&self) -> &[u8; Memory::MEMORY_SIZE] {
        self.ram.get_content()
    }

    pub fn set_register(&mut self, x: usize, value: u8) {
        self.registers[x] = value;
    }

    pub fn set_pc(&mut self, addr: usize) {
        self.pc = addr;
    }
}