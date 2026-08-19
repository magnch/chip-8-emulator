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

use rand;

use crate::config::Config;
use crate::display::Display;
use crate::error::Chip8Error;
use crate::keypad::Keypad;
use crate::memory::Memory;
use crate::opcode;
use crate::opcode::Instruction;
use crate::utils;

/// CPU state struct for use with Chip-8 debugger
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    registers: [u8; Chip8::NUM_REGS],
    stack: [u16; Chip8::STACK_SIZE],
    sp: usize,
    pc: usize,
    index: usize,
    delay_timer: u8,
    sound_timer: u8,
}

/// Chip8 emulator struct containing cpu, registers and peripherals
pub struct Chip8 {
    config: Config,
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
    /// Max number of elements on stack
    pub(crate) const STACK_SIZE: usize = 16;
    /// Number of CPU registers
    pub(crate) const NUM_REGS: usize = 16;

    /// Construct a Chip-8 object with default values
    pub fn new() -> Self {
        Chip8 {
            config: Config::new(),
            ram: Memory::new(),
            display: Display::new(),
            keypad: Keypad::new(),
            registers: [0; Self::NUM_REGS],
            stack: [0; Self::STACK_SIZE],
            sp: 0,
            pc: 0,
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
    /// Load ROM into memory from an array slice
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        self.ram.load_rom(rom)?;
        self.pc = Memory::ROM_START_ADDR;
        Ok(())
    }
    /// Step through one CPU cycle
    pub fn step(&mut self) -> Result<(), Chip8Error> {
        let opcode = self.fetch()?;
        let instruction = self.decode(opcode)?;
        self.execute(instruction)
    }
    /// Decrement delay and sound timers
    pub fn tick_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }
    /// Get display handle
    pub fn get_display(&self) -> &Display {
        &self.display
    }
    /// Get memory handle
    pub fn get_memory(&self) -> &Memory {
        &self.ram
    }
    /// Get CPU state
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
    /// Register key press
    pub fn key_down(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.keypad.press_key(key)
    }
    /// Register key release
    pub fn key_up(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.keypad.release_key(key)
    }
    /// Check if buzzer is beeping (sound timer > 0)
    pub fn is_beeping(&self) -> bool {
        self.sound_timer > 0
    }
    /// Fetch next opcode from memory
    fn fetch(&mut self) -> Result<u16, Chip8Error> {
        // Fetch opcode from PC
        let high_byte = self.ram.read(self.pc)? as u16;
        let low_byte = self.ram.read(self.pc + 1)? as u16;
        // Increment PC
        self.pc += 2;
        // Return 16-bit opcode
        Ok((high_byte << 8) | low_byte)
    }
    /// Decode opcode into instruction
    fn decode(&self, opcode: u16) -> Result<Instruction, Chip8Error> {
        match opcode::decode(opcode) {
            Instruction::Unknown => Err(Chip8Error::UnknownOpcode(opcode)),
            instruction => Ok(instruction),
        }
    }
    /// Execute instruction
    fn execute(&mut self, instruction: Instruction) -> Result<(), Chip8Error> {
        match instruction {
            Instruction::Cls => self.display.clear(),
            Instruction::Rts => self.pc = self.pop()? as usize,
            Instruction::Low => (),
            Instruction::High => (),
            Instruction::Jmp(nnn) => self.pc = nnn,
            Instruction::Jsr(nnn) => self.execute_jsr(nnn)?,
            Instruction::SkeqConst(x, nn) => self.skip_if_eq(self.registers[x], nn),
            Instruction::SkneConst(x, nn) => self.skip_if_not_eq(self.registers[x], nn),
            Instruction::Skeq(x, y) => self.skip_if_eq(self.registers[x], self.registers[y]),
            Instruction::MovConst(x, nn) => self.registers[x] = nn,
            Instruction::AddConst(x, nn) => self.registers[x] = self.registers[x].wrapping_add(nn),
            Instruction::Mov(x, y) => self.registers[x] = self.registers[y],
            Instruction::Or(x, y) => self.registers[x] |= self.registers[y],
            Instruction::And(x, y) => self.registers[x] &= self.registers[y],
            Instruction::Xor(x, y) => self.registers[x] ^= self.registers[y],
            Instruction::Add(x, y) => self.execute_add(x, y),
            Instruction::Sub(x, y) => self.execute_sub(x, y),
            Instruction::Shr(x, y) => self.execute_shr(x, y),
            Instruction::Rsb(x, y) => self.execute_rsb(x, y),
            Instruction::Shl(x, y) => self.execute_shl(x, y),
            Instruction::Skne(x, y) => self.skip_if_not_eq(self.registers[x], self.registers[y]),
            Instruction::Mvi(nnn) => self.index = nnn,
            Instruction::Jmi(nnn) => self.execute_jmi(nnn),
            Instruction::Rand(x, nn) => self.rand(x, nn),
            Instruction::Sprite(x, y, n) => self.execute_sprite(x, y, n)?,
            Instruction::Skpr(x) => if self.keypad.is_pressed(self.registers[x] as usize)? {self.pc += 2},
            Instruction::Skup(x) => if !self.keypad.is_pressed(self.registers[x] as usize)? {self.pc += 2},
            Instruction::Gdelay(x) => self.registers[x] = self.delay_timer,
            Instruction::Key(x) => self.execute_key(x),
            Instruction::Sdelay(x) => self.delay_timer = self.registers[x],
            Instruction::Ssound(x) => self.sound_timer = self.registers[x],
            Instruction::Adi(x) => self.execute_adi(x),
            Instruction::Font(x) => self.execute_font(x),
            Instruction::Xfont(_x) => (),
            Instruction::Bcd(x) => self.execute_bcd(x)?,
            Instruction::Str(x) => self.execute_str(x)?,
            Instruction::Ldr(x) => self.execute_ldr(x)?,

            Instruction::Unknown => unreachable!("tried to execute Instruction::Unknown"),
        }
        Ok(())
    }
    /// Set VF register to value
    fn set_vf(&mut self, value: u8) {
        self.registers[0xF] = value;
    }
    /// Push element to stack
    fn push(&mut self, value: u16) -> Result<(), Chip8Error> {
        if self.sp >= Self::STACK_SIZE {
            return Err(Chip8Error::StackOverflow);
        }
        self.stack[self.sp] = value;
        self.sp += 1;
        Ok(())
    }
    /// Pop element from stack
    fn pop(&mut self) -> Result<u16, Chip8Error> {
        if self.sp == 0 {
            return Err(Chip8Error::StackUnderflow);
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }
}


// CPU execute helper functions
impl Chip8 {
    /// Execute Jsr instruction
    fn execute_jsr(&mut self, nnn: usize) -> Result<(), Chip8Error> {
        self.push(self.pc as u16)?;
        self.pc = nnn;
        Ok(())
    }
    /// Skip next instruction if x equals y
    fn skip_if_eq(&mut self, x: u8, y: u8) {
        if x == y {
            self.pc += 2;
        }
    }
    /// Skip next instruction if x does not equal y
    fn skip_if_not_eq(&mut self, x: u8, y: u8) {
        if x != y {
            self.pc += 2;
        }
    }
    /// Execute Add instruction
    fn execute_add(&mut self, x: usize, y: usize) {
        let (result, carry) = utils::add_with_carry(self.registers[x], self.registers[y]);
        self.registers[x] = result;
        self.set_vf(carry);
    }
    /// Execute Sub instruction
    fn execute_sub(&mut self, x: usize, y: usize) {
        let (result, borrow) = utils::sub_with_borrow(self.registers[x], self.registers[y]);
        self.registers[x] = result;
        self.set_vf(borrow);
    }
    /// Execute Rsb instruction
    fn execute_rsb(&mut self, x: usize, y: usize){
        let (result, borrow) = utils::sub_with_borrow(self.registers[y], self.registers[x]);
        self.registers[x] = result;
        self.set_vf(borrow);
    }
    /// Execute Shr instruction
    fn execute_shr(&mut self, x: usize, y: usize) {
        if self.config.shift_uses_vy {
            self.registers[x] = self.registers[y];
        }
        // Store bit 0 in VF
        let carry = utils::extract_bit(self.registers[x], 0);
        self.registers[x] >>= 1;
        self.set_vf(carry);
    }
    /// Execute Shl instruction
    fn execute_shl(&mut self, x: usize, y: usize) {
        if self.config.shift_uses_vy {
            self.registers[x] = self.registers[y];
        }
        // Store bit 7 in VF
        let carry = utils::extract_bit(self.registers[x], 7);
        self.registers[x] <<= 1;
        self.set_vf(carry);
    }
    /// Execute Jmi instruction
    fn execute_jmi(&mut self, nnn: usize) {
        let offset = if self.config.jmi_uses_vx {
            let x = utils::extract_nibbles(nnn as u16, 2, 1) as usize;
            self.registers[x] as usize
        } else {
            self.registers[0] as usize
        };
        self.pc = nnn + offset;
    }
    /// Generate random number and load into register
    fn rand(&mut self, x: usize, nn: u8) {
        let rand_num: u8 = rand::random();
        self.registers[x] = rand_num & nn;
    }
    /// Execute Sprite instruction
    fn execute_sprite(&mut self, x: usize, y: usize, n: u8) -> Result<(), Chip8Error> {
        let x_coord = self.registers[x] as usize;
        let y_coord = self.registers[y] as usize;
        // Wrap start coordinate, but not rest of sprite
        let x_coord = x_coord % Display::WIDTH;
        let y_coord = y_coord % Display::HEIGHT;

        self.set_vf(0);
        let sprite = self.ram.read_slice(self.index, n as usize)?;
        let collision = self.display.draw_sprite(x_coord, y_coord, sprite)?;
        if collision {
            self.set_vf(1);
        }
        Ok(())
    }
    /// Execute Key instruction
    fn execute_key(&mut self, x: usize) {
        let (key, pressed) = self.keypad.is_pressed_any();
        if pressed {
            self.registers[x] = key as u8;
        } else {
            self.pc -= 2; // Stay at same instruction
        }
    }
    /// Execute Adi instruction
    fn execute_adi(&mut self, x: usize) {
        let mut value = self.index + self.registers[x] as usize;
        if value > 0x0FFF {
            value &= 0x0FFF;
            if self.config.adi_flags_overflow {
                self.set_vf(1);
            }
        }
        self.index = value;
    }
    /// Execute Font instruction
    fn execute_font(&mut self, x: usize) {
        let hex_char: u8 = self.registers[x] & 0x0F;
        let address = Memory::FONT_START_ADDR + (hex_char as usize) * Memory::FONT_CHAR_SIZE;
        self.index = address;
    }
    /// Execute Bcd instruction
    fn execute_bcd(&mut self, x:usize) -> Result<(), Chip8Error> {
        let num = self.registers[x];
        let digits = &[num/100, (num % 100) / 10, num % 10];
        self.ram.write_slice(self.index, digits, 3)
    }
    /// Execute Str instruction
    fn execute_str(&mut self, x: usize) -> Result<(), Chip8Error> {
        for i in 0..=x {
            if self.config.str_ldr_increments_index {
                self.ram.write(self.index, self.registers[i])?;
                self.index += 1;
            } else {
                self.ram.write(self.index + i, self.registers[i])?;
            }    
        }
        Ok(())
    }
    /// Execute Ldr instruction
    fn execute_ldr(&mut self, x: usize) -> Result<(), Chip8Error> {
        for i in 0..=x {
            if self.config.str_ldr_increments_index {
                self.registers[i] = self.ram.read(self.index)?;
                self.index += 1;
            } else {
                self.registers[i] = self.ram.read(self.index + i)?;
            }    
        }
        Ok(())
    }
}
