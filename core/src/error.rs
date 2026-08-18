use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip8Error { 
    UnknownOpcode(u16),
    StackOverflow,
    StackUnderflow,
    RomTooLarge{size: usize, max_size: usize},
    MemoryOutOfBounds{address: usize},
    DisplayOutOfBounds{row: usize, col: usize},
    KeypadOutOfBounds{key: usize},
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chip8Error::UnknownOpcode(opcode) => {
                write!(f, "unknown opcode: {opcode:#06X}")
            }
            Chip8Error::StackOverflow => {
                write!(f, "stack overflow: exceeded 16 nested calls")
            }
            Chip8Error::StackUnderflow => {
                write!(f, "stack underflow: RET with no matching CALL")
            }
            Chip8Error::RomTooLarge { size, max_size } => {
                write!(f, "ROM too large: {size} bytes, max is {max_size} bytes")
            }
            Chip8Error::MemoryOutOfBounds { address } => {
                write!(f, "out of bounds memory access at address: {address:#06X}")
            }
            Chip8Error::DisplayOutOfBounds { row, col } => {
                write!(f, "out of bounds display access at row {row}, column {col}")
            }
            Chip8Error::KeypadOutOfBounds { key } => {
                write!(f, "out of bounds keypad key: {key}")
            }
        }
    }
}

impl std::error::Error for Chip8Error {}