/// Extracts `length` nibbles from `number`, starting `position` nibbles from the right (LSB).
///
/// e.g. `extract_nibbles(0xABCD, 2, 1)` returns `0xB` (the third nibble from the right).
pub(crate) fn extract_nibbles(number: u16, position: u8, length: u8) -> u16 {
    let shift = (position as u32) * 4;
    let mask = if length == 0 {
        0
    } else {
        ((1u32 << (length as u32 * 4)) - 1) as u16
    };

    (number as u32 >> shift) as u16 & mask
}

/// Extracts bit from number in `position`, counting from LSB
pub(crate) fn extract_bit(byte: u8, position: u8) -> u8 {
    (byte >> position) & 1
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nibbles() {
        let test_num: u16 = 0xABCD;
        assert_eq!(extract_nibbles(test_num, 1, 1), 0x000C);
        assert_eq!(extract_nibbles(test_num, 2, 3), 0x00AB);
        assert_eq!(extract_nibbles(test_num, 0, 4), test_num);
        assert_eq!(extract_nibbles(test_num, 0, 0), 0x0000);
    }

    #[test]
    fn test_extract_bit() {
        let test_byte = 0b01010101;
        assert_eq!(extract_bit(test_byte, 0), 1);
        assert_eq!(extract_bit(test_byte, 1), 0);
    }

    #[test]
    fn test_add_with_carry () {
        assert_eq!(add_with_carry(255, 2), (1, 1));
        assert_eq!(add_with_carry(0, 1), (1, 0));
    }

    #[test]
    fn test_sub_with_borrow() {
        assert_eq!(sub_with_borrow(255, 255), (0, 1));
        assert_eq!(sub_with_borrow(250, 255), (251, 0));
    }
}