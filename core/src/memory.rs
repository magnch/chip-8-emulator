use crate::error::Chip8Error;

/// The 4 KiB memory space used by a CHIP-8 program.
pub struct Memory {
    content: [u8; Self::MEMORY_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub const MEMORY_SIZE: usize = 4096;
    pub(crate) const FONT_START_ADDR: usize = 0x050;
    const FONT_END_ADDR: usize = 0x09F;
    pub(crate) const FONT_CHAR_SIZE: usize = 5;
    pub(crate) const ROM_START_ADDR: usize = 0x200;

    /// Create memory and load the built-in hexadecimal font.
    pub(crate) fn new() -> Self {
        let mut memory = Memory {
            content: [0; Self::MEMORY_SIZE],
        };
        memory.load_font();
        memory
    }

    /// Read one byte from memory.
    pub(crate) fn read(&self, address: usize) -> Result<u8, Chip8Error> {
        if self.out_of_bounds(address) {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            Ok(self.content[address])
        }
    }

    /// Write one byte to memory.
    pub(crate) fn write(&mut self, address: usize, data: u8) -> Result<(), Chip8Error> {
        if self.out_of_bounds(address) {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            self.content[address] = data;
            Ok(())
        }
    }

    /// Read a contiguous range of memory.
    pub(crate) fn read_slice(&self, address: usize, length: usize) -> Result<&[u8], Chip8Error> {
        let end = address + length;
        if end > Self::MEMORY_SIZE {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            Ok(&self.content[address..address + length])
        }
    }

    /// Write a contiguous range of memory.
    pub(crate) fn write_slice(
        &mut self,
        address: usize,
        data: &[u8],
        length: usize,
    ) -> Result<(), Chip8Error> {
        let end = address + length;
        if end > Self::MEMORY_SIZE {
            Err(Chip8Error::MemoryOutOfBounds { address })
        } else {
            self.content[address..address + length].copy_from_slice(data);
            Ok(())
        }
    }

    /// Fill memory with 0's
    pub(crate) fn clear(&mut self) {
        self.content.fill(0);
    }

    /// Copy a ROM into program memory starting at `0x200`.
    pub(crate) fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        let rom_end_addr = Self::ROM_START_ADDR + rom.len();
        let max_size = Self::MEMORY_SIZE - Self::ROM_START_ADDR;

        if self.out_of_bounds(rom_end_addr - 1) {
            Err(Chip8Error::RomTooLarge {
                size: (rom.len()),
                max_size: (max_size),
            })
        } else {
            self.content[Self::ROM_START_ADDR..rom_end_addr].copy_from_slice(rom);
            Ok(())
        }
    }

    /// Load the standard CHIP-8 hexadecimal font into memory.
    pub(crate) fn load_font(&mut self) {
        let font = crate::font::FONT_SET;
        self.content[Self::FONT_START_ADDR..=Self::FONT_END_ADDR].copy_from_slice(&font);
    }

    /// Check whether an address is outside the 4 KiB memory space.
    fn out_of_bounds(&self, address: usize) -> bool {
        address >= Self::MEMORY_SIZE
    }
}

#[cfg(feature = "debug-tools")]
impl Memory {
    pub(crate) fn get_content(&self) -> &[u8; Self::MEMORY_SIZE] {
        &self.content
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn test_read_write() {
        let mut memory = Memory::new();
        for i in 0..Memory::MEMORY_SIZE {
            let write_value: u8 = (i % 256) as u8;
            memory
                .write(i, write_value)
                .expect("write to valid address should succeed");
            let read_value = memory
                .read(i)
                .expect("read from valid address should succeed");

            assert_eq!(read_value, write_value);
        }
    }

    #[test]
    fn test_read_write_slice() {
        let mut memory = Memory::new();
        let write_data: [u8; Memory::MEMORY_SIZE] = [0xAB; Memory::MEMORY_SIZE];
        memory
            .write_slice(0, &write_data, Memory::MEMORY_SIZE)
            .expect("tried to write to valid address range");
        let read_data = memory
            .read_slice(0, Memory::MEMORY_SIZE)
            .expect("tried to read from valid address range");

        assert_eq!(*read_data, write_data);
    }

    #[test]
    fn test_write_empty_slice() {
        let mut memory = Memory::new();
        let write_data: [u8; 0] = [];
        memory
            .write_slice(0, &write_data, 0)
            .expect("should handle empty slices");
    }

    #[test]
    fn test_out_of_bounds() {
        let mut memory = Memory::new();
        assert_eq!(memory.out_of_bounds(0), false);
        assert_eq!(memory.out_of_bounds(4096), true);
        assert!(memory.read(4096).is_err());
        assert!(memory.write(4096, 0).is_err());
        assert!(memory.read_slice(4094, 3).is_err());
        assert!(memory.write_slice(4094, &[0, 1, 2], 3).is_err());
    }

    #[test]
    fn test_load_font() {
        let mut memory = Memory::new();
        memory.load_font();
        let font = crate::font::FONT_SET;
        assert_eq!(
            font,
            memory
                .read_slice(Memory::FONT_START_ADDR, crate::font::FONT_SET_SIZE)
                .expect("")
        );
    }

    #[test]
    fn test_load_rom() {
        let mut memory = Memory::new();
        let rom: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let oversized_rom: [u8; 5000] = [0xFF; 5000];

        memory.load_rom(&rom).expect("ROM size within limits");
        assert_eq!(
            memory.read_slice(Memory::ROM_START_ADDR, 10).expect(""),
            &rom
        );

        assert!(memory.load_rom(&oversized_rom).is_err());
    }
}
