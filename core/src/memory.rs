pub(crate) struct Memory {
    content: [u8; Self::MEMORY_SIZE],
}

impl Memory {
    
    const MEMORY_SIZE: usize = 4096;

    pub(crate) fn read(&self, address: usize) -> u8 {
        self.content[address]
    }

    pub(crate) fn write(&mut self, address: usize, data: u8) {
        self.content[address] = data;
    }

    pub(crate) fn read_slice(&self, address: usize, length: usize) -> &[u8] {
        &self.content[address..address+length]
    }
}