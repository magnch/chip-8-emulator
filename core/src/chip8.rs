//! # Chip8 emulator core module
//! 
//! This module provides the Chip8 struct for running the emulator core logic
//! and interfacing with the main application
//! 
//! ## Examples
//! ```
//! use chip8_core::chip8;
//! let mut chip8 = chip8::Chip8::new();
//! ```

use crate::display::Display;
use crate::memory::Memory;
use crate::keypad::Keypad;
use crate::opcode;
use crate::opcode::Instruction;
use rand;

/// Chip8 emulator struct containing cpu, registers and peripherals
pub struct Chip8 {
    ram: Memory,
    display: Display,
    keypad: Keypad,
    registers: [u8; Self::NUM_REGS],
    stack: [u16; Self::STACK_SIZE],
    sp: usize,
    pc: usize,
    index: usize,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    const STACK_SIZE: usize = 16;
    const NUM_REGS: usize = 16;

    pub fn new() -> Self {
        Chip8 {
            ram: Memory::new(),
            display: Display::new(),
            keypad: Keypad::new(),
            registers: [0; Self::NUM_REGS],
            stack: [0; Self::STACK_SIZE],
            sp: 1,
            pc: 0,
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram.load_rom(rom);
        self.pc = Memory::ROM_START_ADDR;
    }

    pub fn step(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }

    pub fn tick_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn get_display(&self) -> &Display {
        &self.display
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
            Instruction::Rts => self.pc = self.pop() as usize,
            Instruction::Low => (),
            Instruction::High => (),
            Instruction::Jmp(nnn) => self.pc = nnn,
            Instruction::Jsr(nnn) => self.execute_jsr(nnn),
            Instruction::SkeqConst(x, nn) => self.skip_if_eq(self.registers[x], nn),
            Instruction::SkneConst(x, nn) => self.skip_if_not_eq(self.registers[x], nn),
            Instruction::Skeq(x, y) => self.skip_if_eq(self.registers[x], self.registers[y]),
            Instruction::MovConst(x, nn) => self.registers[x] = nn,
            Instruction::AddConst(x, nn) => self.registers[x] = self.registers[x].wrapping_add(nn),
            Instruction::Mov(x, y) => self.registers[x] = self.registers[y],
            Instruction::Or(x, y) => self.registers[x] |= self.registers[y],
            Instruction::And(x, y) => self.registers[x] &= self.registers[y],
            Instruction::Xor(x, y) => self.registers[x] ^= self.registers[y],
            Instruction::Add(x, y) => self.registers[x] = self.add_with_carry(x, y),
            Instruction::Sub(x, y) => self.registers[x] = self.sub_with_carry(x, y),
            Instruction::Shr(x) => self.shift_right(x),
            Instruction::Rsb(x, y) => self.registers[x] = self.sub_with_carry(y, x),
            Instruction::Shl(x) => self.shift_left(x),
            Instruction::Skne(x, y) => self.skip_if_not_eq(self.registers[x], self.registers[y]),
            Instruction::Mvi(nnn) => self.index = nnn,
            Instruction::Jmi(nnn) => self.pc = nnn + self.registers[0] as usize,
            Instruction::Rand(x, nn) => self.rand(x, nn),
            Instruction::Sprite(x, y, n) => self.execute_sprite(x, y, n),
            Instruction::Skpr(x) => if self.keypad.is_pressed(x) {self.pc += 2},
            Instruction::Skup(x) => if !self.keypad.is_pressed(x) {self.pc += 2},
            Instruction::Gdelay(x) => self.registers[x] = self.delay_timer,
            Instruction::Key(x) => self.execute_key(x),
            Instruction::Sdelay(x) => self.delay_timer = self.registers[x],
            Instruction::Ssound(x) => self.sound_timer = self.registers[x],
            Instruction::Adi(x) => self.index += self.registers[x] as usize,
            Instruction::Font(x) => self.execute_font(x),
            Instruction::Xfont(_x) => (),
            Instruction::Bcd(x) => self.execute_bcd(x),
            Instruction::Str(x) => self.execute_str(x),
            Instruction::Ldr(x) => self.execute_ldr(x),

            Instruction::None => panic!("Tried to execute Instruction::None!"),
            _ => panic!("Tried to execute unsupported instruction!"),
        }
    }

    fn set_vf(&mut self, value: u8) {
        self.registers[0xF] = value;
    }

    fn push(&mut self, value: u16) {
        if self.sp < Self::STACK_SIZE {
            self.stack[self.sp] = value;
            self.sp += 1;
        }
    }

    fn pop(&mut self) -> u16 {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp];
        }
        0
    }
}


// CPU execute helper functions
impl Chip8 {
    fn execute_jsr(&mut self, nnn: usize) {
        self.push(self.pc as u16);
        self.pc = nnn;
    }

    fn skip_if_eq(&mut self, x: u8, y: u8) {
        if x == y {
            self.pc += 2;
        }
    }

    fn skip_if_not_eq(&mut self, x: u8, y: u8) {
        if x != y {
            self.pc += 2;
        }
    }

    fn add_with_carry(&mut self, x: usize, y: usize) -> u8 {
        let (result, carry) = self.registers[x].overflowing_add(self.registers[y]);
        self.set_vf(carry as u8);
        result
    }

    fn sub_with_carry(&mut self, x: usize, y: usize) -> u8 {
        let (result, carry) = self.registers[x].overflowing_sub(self.registers[y]);
        self.set_vf(!carry as u8);
        result
    }

    fn shift_right(&mut self, x: usize) {
        // Store bit 0 in VF
        self.set_vf((x & 0x01) as u8);
        self.registers[x] >>= 1;
    }

    fn shift_left(&mut self, x: usize) {
        // Store bit 7 in VF
        self.set_vf((x & 0x80) as u8);
        self.registers[x] <<= 1;
    }

    fn rand(&mut self, x: usize, nn: u8) {
        let rand_num: u8 = rand::random();
        self.registers[x] = rand_num & nn;
    }

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

    fn execute_key(&mut self, x: usize) {
        let (key, pressed) = self.keypad.is_pressed_any();
        if pressed {
            self.registers[x] = key as u8;
        } else {
            self.pc -= 2; // Stay at same instruction
        }
    }

    fn execute_font(&mut self, x: usize) {
        let hex_char: u8 = self.registers[x] & 0x0F;
        let address = Memory::FONT_START_ADDR + (hex_char as usize) * Memory::FONT_CHAR_SIZE;
        self.index = address;
    }

    fn execute_bcd(&mut self, x:usize) {
        let num = self.registers[x];
        let digits = &[num/100, (num % 100) / 10, num % 10];
        self.ram.write_slice(self.index, digits, 3);
    }

    fn execute_str(&mut self, x: usize) {
        for i in 0..=x {
            self.ram.write(self.index + i, self.registers[i])
        } 
    }

    fn execute_ldr(&mut self, x: usize) {
        for i in 0..=x {
            self.registers[i] = self.ram.read(self.index + i)
        } 
    }
}
