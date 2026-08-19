/// Extracts `length` nibbles from `opcode`, starting `position` nibbles from the right (LSB).
///
/// e.g. `extract_nibbles(0xABCD, 2, 1)` returns `0xB` (the third nibble from the right).
pub(crate) fn extract_nibbles(opcode: u16, position: u8, length: u8) -> u16 {
    (opcode >> (4 * position)) & ((1 << (4 * length)) - 1)
}

/// Extracts bit from number in `position`, counting from LSB
pub(crate) fn extract_bit(number: u8, position: u8) -> u8 {
    (number & (1 << position)) >> position
} 

/// Add x and y, and return result and carry
pub(crate) fn add_with_carry(lhs: u8, rhs: u8) -> (u8, u8) {
    let (result, overflow) = lhs.overflowing_add(rhs);
    (result, overflow as u8)
}
// Substract y from x, and return result and borrow
pub(crate) fn sub_with_borrow(lhs: u8, rhs: u8) -> (u8, u8) {
    let (result, overflow) = lhs.overflowing_sub(rhs);
    (result, !overflow as u8)
}