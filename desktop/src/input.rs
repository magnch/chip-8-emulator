use sdl2::keyboard::Scancode;

/// Maps a physical key position to its CHIP-8 keypad index (0x0-0xF), if any.
///
/// Layout (physical position -> CHIP-8 key):
///   1 2 3 4        1 2 3 C
///   Q W E R   ->   4 5 6 D
///   A S D F        7 8 9 E
///   Z X C V        A 0 B F
pub fn map_scancode(scancode: Scancode) -> Option<usize> {
    match scancode {
        // First row
        Scancode::Num1 => Some(0x1), 
        Scancode::Num2 => Some(0x2),
        Scancode::Num3 => Some(0x3), 
        Scancode::Num4 => Some(0xC),
        // Second row
        Scancode::Q => Some(0x4), 
        Scancode::W => Some(0x5),
        Scancode::E => Some(0x6), 
        Scancode::R => Some(0xD),
        // Third row
        Scancode::A => Some(0x7), 
        Scancode::S => Some(0x8),
        Scancode::D => Some(0x9), 
        Scancode::F => Some(0xE),
        // Fourth row
        Scancode::Z => Some(0xA), 
        Scancode::X => Some(0x0),
        Scancode::C => Some(0xB), 
        Scancode::V => Some(0xF),

        _ => None,
    }
}