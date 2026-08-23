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

/// Snapshot of the emulator state for debugger integrations.
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

/// CHIP-8 virtual machine state and instruction executor.
pub struct Chip8 {
    /// Compatibility settings for instruction variants. Public so a
    /// frontend can read or replace it at any time; takes effect starting
    /// with the next instruction executed.
    pub config: Config,
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

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    /// Max number of elements on stack
    pub(crate) const STACK_SIZE: usize = 16;
    /// Number of CPU registers
    pub(crate) const NUM_REGS: usize = 16;

    /// Create a reset CHIP-8 machine.
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

    /// Load a ROM and reset the program counter to its start address.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        self.ram.load_rom(rom)?;
        self.pc = Memory::ROM_START_ADDR;
        Ok(())
    }

    /// Fetch, decode, and execute one instruction.
    pub fn step(&mut self) -> Result<(), Chip8Error> {
        let opcode = self.fetch()?;
        let instruction = self.decode(opcode)?;
        self.execute(instruction)
    }

    /// Decrement the delay and sound timers by one tick.
    pub fn tick_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    /// Reset the state of the Chip-8
    pub fn reset(&mut self) {
        self.ram.clear();
        self.display.clear();
        self.registers.fill(0x00);
        self.stack.fill(0x00);
        self.sp = 0;
        self.pc = 0;
        self.index = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.ram.load_font();
    }

    /// Borrow the current display buffer.
    pub fn get_display(&self) -> &Display {
        &self.display
    }

    /// Borrow the emulator memory.
    pub fn get_memory(&self) -> &Memory {
        &self.ram
    }

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

    /// Mark a CHIP-8 key as pressed.
    pub fn key_down(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.keypad.press_key(key)
    }

    /// Mark a CHIP-8 key as released.
    pub fn key_up(&mut self, key: usize) -> Result<(), Chip8Error> {
        self.keypad.release_key(key)
    }

    /// Return whether the sound timer is active.
    pub fn is_beeping(&self) -> bool {
        self.sound_timer > 0
    }

    /// Take dirty bit from display
    pub fn display_take_dirty(&mut self) -> bool {
        self.display.take_dirty()
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
            Instruction::Skpr(x) => {
                if self.keypad.is_pressed(self.registers[x] as usize)? {
                    self.pc += 2
                }
            }
            Instruction::Skup(x) => {
                if !self.keypad.is_pressed(self.registers[x] as usize)? {
                    self.pc += 2
                }
            }
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
    fn execute_rsb(&mut self, x: usize, y: usize) {
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

        // Wrap start coordinates, but not rest of sprite
        let x_coord = x_coord % Display::WIDTH;
        let y_coord = y_coord % Display::HEIGHT;

        self.set_vf(0);
        let sprite = self.ram.read_slice(self.index, n as usize)?;
        let collision =
            self.display
                .draw_sprite(x_coord, y_coord, sprite, self.config.sprites_wrap_at_edge)?;
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
    fn execute_bcd(&mut self, x: usize) -> Result<(), Chip8Error> {
        let num = self.registers[x];
        let digits = &[num / 100, (num % 100) / 10, num % 10];
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_opcode(chip8: &mut Chip8, opcode: u16) {
        chip8
            .load_rom(&[(opcode >> 8) as u8, opcode as u8])
            .unwrap();
    }

    #[test]
    fn test_new_initializes_cpu() {
        let chip8 = Chip8::new();
        let state = chip8.get_state();

        assert_eq!(state.registers, [0; Chip8::NUM_REGS]);
        assert_eq!(state.stack, [0; Chip8::STACK_SIZE]);
        assert_eq!(state.sp, 0);
        assert_eq!(state.pc, 0);
        assert_eq!(state.index, 0);
        assert_eq!(state.delay_timer, 0);
        assert_eq!(state.sound_timer, 0);
    }

    #[test]
    fn test_load_rom_sets_program_counter() {
        let mut chip8 = Chip8::new();

        chip8.load_rom(&[0x60, 0x2A]).unwrap();

        assert_eq!(chip8.pc, Memory::ROM_START_ADDR);
        assert_eq!(chip8.ram.read(Memory::ROM_START_ADDR), Ok(0x60));
        assert_eq!(chip8.ram.read(Memory::ROM_START_ADDR + 1), Ok(0x2A));
    }

    #[test]
    fn test_load_rom_rejects_oversized_rom() {
        let mut chip8 = Chip8::new();
        let max_size = 0x1000 - Memory::ROM_START_ADDR;
        let rom = vec![0xFF; max_size + 1];

        assert_eq!(
            chip8.load_rom(&rom),
            Err(Chip8Error::RomTooLarge {
                size: rom.len(),
                max_size,
            })
        );
    }

    #[test]
    fn test_step_executes_instruction_and_advances_pc() {
        let mut chip8 = Chip8::new();
        load_opcode(&mut chip8, 0x612A);

        chip8.step().unwrap();

        assert_eq!(chip8.registers[1], 0x2A);
        assert_eq!(chip8.pc, Memory::ROM_START_ADDR + 2);
    }

    #[test]
    fn test_step_returns_error_for_unknown_opcode() {
        let mut chip8 = Chip8::new();
        load_opcode(&mut chip8, 0x0000);

        assert_eq!(chip8.step(), Err(Chip8Error::UnknownOpcode(0x0000)));
    }

    #[test]
    fn test_tick_timers_decrements_without_underflow() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer = 2;
        chip8.sound_timer = 1;

        chip8.tick_timers();
        assert_eq!(chip8.delay_timer, 1);
        assert_eq!(chip8.sound_timer, 0);
        assert!(!chip8.is_beeping());

        chip8.tick_timers();
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
    }

    #[test]
    fn test_is_beeping_when_sound_timer_is_nonzero() {
        let mut chip8 = Chip8::new();
        chip8.sound_timer = 1;

        assert!(chip8.is_beeping());
    }

    #[test]
    fn test_jsr_and_rts_use_stack() {
        let mut chip8 = Chip8::new();
        chip8.pc = Memory::ROM_START_ADDR + 2;

        chip8.execute(Instruction::Jsr(0x300)).unwrap();
        assert_eq!(chip8.pc, 0x300);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], (Memory::ROM_START_ADDR + 2) as u16);

        chip8.execute(Instruction::Rts).unwrap();
        assert_eq!(chip8.pc, Memory::ROM_START_ADDR + 2);
        assert_eq!(chip8.sp, 0);
    }

    #[test]
    fn test_stack_overflow_and_underflow() {
        let mut chip8 = Chip8::new();

        assert_eq!(chip8.pop(), Err(Chip8Error::StackUnderflow));
        for value in 0..Chip8::STACK_SIZE {
            chip8.push(value as u16).unwrap();
        }
        assert_eq!(
            chip8.push(Chip8::STACK_SIZE as u16),
            Err(Chip8Error::StackOverflow)
        );
    }

    #[test]
    fn test_add_sets_carry_flag() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 250;
        chip8.registers[2] = 10;

        chip8.execute(Instruction::Add(1, 2)).unwrap();

        assert_eq!(chip8.registers[1], 4);
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_sub_sets_no_borrow_flag() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 3;
        chip8.registers[2] = 5;

        chip8.execute(Instruction::Sub(1, 2)).unwrap();

        assert_eq!(chip8.registers[1], 254);
        assert_eq!(chip8.registers[0xF], 0);
    }

    #[test]
    fn test_cls_clears_display() {
        let mut chip8 = Chip8::new();
        chip8.display.set_pixel(2, 3, true).unwrap();

        chip8.execute(Instruction::Cls).unwrap();

        assert!(!chip8.display.get_content()[2][3]);
    }

    #[test]
    fn test_sprite_sets_collision_flag() {
        let mut chip8 = Chip8::new();
        chip8.index = 0x300;
        chip8.ram.write(0x300, 0b1000_0000).unwrap();
        chip8.registers[1] = 2;
        chip8.registers[2] = 3;

        chip8.execute(Instruction::Sprite(1, 2, 1)).unwrap();
        assert_eq!(chip8.registers[0xF], 0);

        chip8.execute(Instruction::Sprite(1, 2, 1)).unwrap();
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_key_down_and_key_up() {
        let mut chip8 = Chip8::new();

        chip8.key_down(0xA).unwrap();
        assert_eq!(chip8.keypad.is_pressed(0xA), Ok(true));

        chip8.key_up(0xA).unwrap();
        assert_eq!(chip8.keypad.is_pressed(0xA), Ok(false));
        assert_eq!(
            chip8.key_down(0x10),
            Err(Chip8Error::KeypadOutOfBounds { key: 0x10 })
        );
    }

    #[test]
    fn test_bcd_writes_decimal_digits() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 231;
        chip8.index = 0x300;

        chip8.execute(Instruction::Bcd(1)).unwrap();

        assert_eq!(chip8.ram.read_slice(0x300, 3).unwrap(), &[2, 3, 1]);
    }

    #[test]
    fn test_store_and_load_registers() {
        let mut chip8 = Chip8::new();
        chip8.index = 0x300;
        chip8.registers[0] = 0x12;
        chip8.registers[1] = 0x34;

        chip8.execute(Instruction::Str(1)).unwrap();
        chip8.registers[0] = 0;
        chip8.registers[1] = 0;
        chip8.execute(Instruction::Ldr(1)).unwrap();

        assert_eq!(chip8.registers[0], 0x12);
        assert_eq!(chip8.registers[1], 0x34);
    }

    mod execution_tests {
        use super::*;

        mod control_flow {
            use super::*;

            #[test]
            fn test_jmp_sets_program_counter() {
                let mut chip8 = Chip8::new();

                chip8.execute(Instruction::Jmp(0x345)).unwrap();

                assert_eq!(chip8.pc, 0x345);
            }

            #[test]
            fn test_skip_instructions() {
                let mut chip8 = Chip8::new();
                chip8.pc = 0x200;
                chip8.registers[1] = 0x42;
                chip8.registers[2] = 0x42;

                chip8.execute(Instruction::SkeqConst(1, 0x42)).unwrap();
                assert_eq!(chip8.pc, 0x202);
                chip8.execute(Instruction::SkneConst(1, 0x42)).unwrap();
                assert_eq!(chip8.pc, 0x202);
                chip8.execute(Instruction::Skeq(1, 2)).unwrap();
                assert_eq!(chip8.pc, 0x204);
                chip8.execute(Instruction::Skne(1, 2)).unwrap();
                assert_eq!(chip8.pc, 0x204);

                chip8.registers[2] = 0x24;
                chip8.execute(Instruction::SkneConst(1, 0x24)).unwrap();
                assert_eq!(chip8.pc, 0x206);
                chip8.execute(Instruction::Skne(1, 2)).unwrap();
                assert_eq!(chip8.pc, 0x208);
            }
        }

        mod register_operations {
            use super::*;

            #[test]
            fn test_add_const_wraps() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0xFF;

                chip8.execute(Instruction::AddConst(1, 1)).unwrap();

                assert_eq!(chip8.registers[1], 0);
            }

            #[test]
            fn test_register_bitwise_operations() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0b1010_1010;
                chip8.registers[2] = 0b1100_0011;

                chip8.execute(Instruction::Or(1, 2)).unwrap();
                assert_eq!(chip8.registers[1], 0b1110_1011);
                chip8.registers[1] = 0b1010_1010;
                chip8.execute(Instruction::And(1, 2)).unwrap();
                assert_eq!(chip8.registers[1], 0b1000_0010);
                chip8.registers[1] = 0b1010_1010;
                chip8.execute(Instruction::Xor(1, 2)).unwrap();
                assert_eq!(chip8.registers[1], 0b0110_1001);
            }

            #[test]
            fn test_move_and_reverse_subtract() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 3;
                chip8.registers[2] = 8;

                chip8.execute(Instruction::Mov(1, 2)).unwrap();
                assert_eq!(chip8.registers[1], 8);
                chip8.registers[1] = 3;
                chip8.execute(Instruction::Rsb(1, 2)).unwrap();
                assert_eq!(chip8.registers[1], 5);
                assert_eq!(chip8.registers[0xF], 1);
            }

            #[test]
            fn test_shift_right_sets_vf() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0b0000_0011;

                chip8.execute(Instruction::Shr(1, 0)).unwrap();

                assert_eq!(chip8.registers[1], 1);
                assert_eq!(chip8.registers[0xF], 1);
            }

            #[test]
            fn test_shift_left_sets_vf() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0b1000_0001;

                chip8.execute(Instruction::Shl(1, 0)).unwrap();

                assert_eq!(chip8.registers[1], 2);
                assert_eq!(chip8.registers[0xF], 1);
            }
        }

        mod index_and_memory {
            use super::*;

            #[test]
            fn test_mvi_sets_index() {
                let mut chip8 = Chip8::new();

                chip8.execute(Instruction::Mvi(0xABC)).unwrap();

                assert_eq!(chip8.index, 0xABC);
            }

            #[test]
            fn test_adi_adds_register_to_index() {
                let mut chip8 = Chip8::new();
                chip8.index = 0x300;
                chip8.registers[1] = 0x24;

                chip8.execute(Instruction::Adi(1)).unwrap();

                assert_eq!(chip8.index, 0x324);
            }

            #[test]
            fn test_font_sets_character_address() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0x0B;

                chip8.execute(Instruction::Font(1)).unwrap();

                assert_eq!(
                    chip8.index,
                    Memory::FONT_START_ADDR + 0x0B * Memory::FONT_CHAR_SIZE
                );
            }

            #[test]
            fn test_adi_wraps_index_to_twelve_bits() {
                let mut chip8 = Chip8::new();
                chip8.index = 0xFFF;
                chip8.registers[1] = 2;

                chip8.execute(Instruction::Adi(1)).unwrap();

                assert_eq!(chip8.index, 1);
            }
        }

        mod keypad_and_timers {
            use super::*;

            #[test]
            fn test_skip_if_key_pressed() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0xA;
                chip8.key_down(0xA).unwrap();
                chip8.pc = 0x200;

                chip8.execute(Instruction::Skpr(1)).unwrap();

                assert_eq!(chip8.pc, 0x202);
            }

            #[test]
            fn test_skip_if_key_not_pressed() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0xA;
                chip8.pc = 0x200;

                chip8.execute(Instruction::Skup(1)).unwrap();

                assert_eq!(chip8.pc, 0x202);
            }

            #[test]
            fn test_key_waits_for_key_and_stores_key() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 0;
                chip8.pc = 0x202;
                chip8.key_down(0xC).unwrap();

                chip8.execute(Instruction::Key(1)).unwrap();

                assert_eq!(chip8.registers[1], 0xC);
                assert_eq!(chip8.pc, 0x202);
            }

            #[test]
            fn test_key_repeats_when_no_key_is_pressed() {
                let mut chip8 = Chip8::new();
                chip8.pc = 0x202;

                chip8.execute(Instruction::Key(1)).unwrap();

                assert_eq!(chip8.pc, 0x200);
            }

            #[test]
            fn test_delay_and_sound_timer_instructions() {
                let mut chip8 = Chip8::new();
                chip8.registers[1] = 7;

                chip8.execute(Instruction::Sdelay(1)).unwrap();
                assert_eq!(chip8.delay_timer, 7);
                chip8.execute(Instruction::Ssound(1)).unwrap();
                assert_eq!(chip8.sound_timer, 7);
                chip8.delay_timer = 3;
                chip8.execute(Instruction::Gdelay(1)).unwrap();
                assert_eq!(chip8.registers[1], 3);
            }
        }

        mod system_and_random {
            use super::*;

            #[test]
            fn test_low_high_and_xfont_are_no_ops() {
                let mut chip8 = Chip8::new();
                chip8.pc = 0x200;
                chip8.index = 0x345;

                chip8.execute(Instruction::Low).unwrap();
                chip8.execute(Instruction::High).unwrap();
                chip8.execute(Instruction::Xfont(1)).unwrap();

                assert_eq!(chip8.pc, 0x200);
                assert_eq!(chip8.index, 0x345);
            }

            #[test]
            fn test_rand_masks_result() {
                let mut chip8 = Chip8::new();

                for _ in 0..100 {
                    chip8.execute(Instruction::Rand(1, 0x0F)).unwrap();
                    assert_eq!(chip8.registers[1] & 0xF0, 0);
                }
            }
        }
    }
}
