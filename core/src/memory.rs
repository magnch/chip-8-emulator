use crate::error::Chip8Error;

pub struct Memory {
    content: [u8; Self::MEMORY_SIZE],
}

impl Memory {
    const MEMORY_SIZE: usize = 4096;
    pub(crate) const FONT_START_ADDR: usize = 0x050;
    const FONT_END_ADDR: usize = 0x09F;
    pub(crate) const FONT_CHAR_SIZE:usize = 5;
    pub(crate) const ROM_START_ADDR: usize = 0x200;

    pub(crate) fn new() -> Self {
        let mut memory = Memory{ content: [0; Self::MEMORY_SIZE] };
        memory.load_font();
        memory
    }

    pub(crate) fn read(&self, address: usize) -> Result<u8, Chip8Error> {
        if self.out_of_bounds(address) {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            Ok(self.content[address])
        }
    }

    pub(crate) fn write(&mut self, address: usize, data: u8) -> Result<(), Chip8Error> {
        if self.out_of_bounds(address) {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            self.content[address] = data;
            Ok(())
        }
    }

    pub(crate) fn read_slice(&self, address: usize, length: usize) -> Result<&[u8], Chip8Error> {
        if self.out_of_bounds(address + length){
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            Ok(&self.content[address..address+length])
        }
    }

    pub(crate) fn write_slice(&mut self, address: usize, data: &[u8], length: usize) -> Result<(), Chip8Error> {
        if self.out_of_bounds(address) {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            self.content[address..address + length].copy_from_slice(data);
            Ok(())
        }
    }

    pub(crate) fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        let rom_end_addr = Self::ROM_START_ADDR + rom.len();
        let max_size = Self::MEMORY_SIZE - Self::ROM_START_ADDR;

        if self.out_of_bounds(rom_end_addr) {
            Err(Chip8Error::RomTooLarge { size: (rom.len()), max_size: (max_size) })
        } else {
            self.content[Self::ROM_START_ADDR..rom_end_addr].copy_from_slice(rom);
            Ok(())
        }
        
    }

    fn load_font(&mut self) {
        let font = crate::font::FONT_SET;
        self.content[Self::FONT_START_ADDR..=Self::FONT_END_ADDR].copy_from_slice(&font);
    }

    fn out_of_bounds(&self, address: usize) -> bool {
        address >= Self::MEMORY_SIZE
    }
}