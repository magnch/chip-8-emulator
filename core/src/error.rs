use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Errors returned by the CHIP-8 core.
pub enum Chip8Error { 
    /// The opcode is not supported.
    UnknownOpcode(u16),
    /// A subroutine call exceeded the stack capacity.
    StackOverflow,
    /// A return was attempted with an empty stack.
    StackUnderflow,
    /// The ROM is larger than the available program memory.
    RomTooLarge{size: usize, max_size: usize},
    /// An address was outside the 4 KiB memory space.
    MemoryOutOfBounds{address: usize},
    /// A display coordinate was outside the framebuffer.
    DisplayOutOfBounds{row: usize, col: usize},
    /// A keypad value was outside the 16-key range.
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