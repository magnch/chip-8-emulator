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

    fn step(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    fn fetch(&mut self) -> u16 {
        // Fetch opcode from PC
        let high_byte = self.ram.read(self.pc) as u16;
        let low_byte = self.ram.read(self.pc + 1) as u16;
        // Increment PC
        self.pc += 2;
        // Return 16-bit opcode
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
            Instruction::Sprite(x, y, n) => self.execute_sprite(x, y, n),

            Instruction::None => panic!("Tried to execute Instruction::None!"),
            _ => panic!("Tried to execute unsupported instruction!"),
        }
    }

    fn set_vf(&mut self, value: u8) {
        self.registers[0xF] = value;
    }
}


// CPU execute helper functions
impl Cpu {
    fn execute_sprite(&mut self, x: usize, y: usize, n: u8) {
        let x_coord = self.registers[x] as usize;
        let y_coord = self.registers[y] as usize;
        // Wrap start coordinate, but not rest of sprite
        let x_coord = x_coord % Display::WIDTH;
        let y_coord = y_coord % Display::HEIGHT;

        self.set_vf(0);
        let sprite = self.ram.read_slice(self.index as usize, n as usize);
        let collision = self.display.draw_sprite(x_coord, y_coord, sprite);
        if collision {
            self.set_vf(1);
        }
    }
}