pub(crate) struct Memory {
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

    pub(crate) fn read(&self, address: usize) -> u8 {
        self.content[address]
    }

    pub(crate) fn write(&mut self, address: usize, data: u8) {
        self.content[address] = data;
    }

    pub(crate) fn read_slice(&self, address: usize, length: usize) -> &[u8] {
        &self.content[address..address+length]
    }

    pub(crate) fn write_slice(&mut self, address: usize, data: &[u8], length: usize) {
        self.content[address..address + length].copy_from_slice(data);
    }

    pub(crate) fn load_rom(&mut self, rom: &[u8]) {
        let rom_end_addr = Self::ROM_START_ADDR + rom.len();
        self.content[Self::ROM_START_ADDR..rom_end_addr].copy_from_slice(rom); 
    }

    fn load_font(&mut self) {
        let font = crate::font::FONT_SET;
        self.content[Self::FONT_START_ADDR..=Self::FONT_END_ADDR].copy_from_slice(&font);
    }
}