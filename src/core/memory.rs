struct Memory {
    content: [u8; Self::MEMORY_SIZE],
}

impl Memory {
    
    const MEMORY_SIZE: usize = 4096;

    fn read(&self, address: usize) -> Option<u8> {
        if address < Self::MEMORY_SIZE {
            Some(self.content[address])
        } else {
            None
        }
    }

    fn write(&mut self, address: usize, data: u8) -> Result<(), String> {
        if address < Self::MEMORY_SIZE {
            self.content[address] = data;
            if self.content[address] == data {
                return Ok(())
            } else {
                return Err("Failed to write data to memory".to_string())
            }
        } else {
            Err("Address out of range: {address}".to_string())
        }
    }
}