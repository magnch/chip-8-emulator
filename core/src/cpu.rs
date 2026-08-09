use crate::display::Display;
use crate::memory::Memory;
use crate::opcode;
use crate::opcode::Instruction;

struct Cpu {
    ram: Memory,
    display: Display,
    stack: [u16; Self::STACK_SIZE],
    registers: [u8; Self::NUM_REGS],
    pc: usize,
    index: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Cpu {
    const STACK_SIZE: usize = 16;
    const NUM_REGS: usize = 16;

    fn fetch(&self) -> u16 {
        let high_byte = self.ram.read(self.pc) as u16;
        let low_byte = self.ram.read(self.pc + 1) as u16;
        (high_byte << 8) | low_byte
    }

    fn decode(&self, opcode: u16) -> Instruction {
        opcode::decode(opcode)
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Cls => self.display.clear(),
            Instruction::Jmp(nnn) => self.pc = nnn as usize,
            Instruction::Mov(x, nn) => self.registers[x] = nn,
            Instruction::Add(x, nn) => self.registers[x] += nn,
            Instruction::Mvi(nnn) => self.index = nnn,

            Instruction::None => panic!("Tried to execute Instruction::None!"),
            _ => panic!("Tried to execute unsupported instruction!"),
        }
    }
}