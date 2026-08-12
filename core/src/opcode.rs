pub(crate) enum Instruction {
    None,
    Cls,                        //00E0
    Jmp(u16),                   //1nnn
    Mov(usize, u8),            //6xnn
    Add(usize, u8),            //7xnn
    Mvi(u16),                   //Annn
    Sprite(usize, usize, u8),   //Dxyn
}

fn extract_nibbles(opcode: u16, position: u8, length: u8) -> u16 {
    (opcode >> (4 * position)) & ((1 << (4 * length)) - 1)
}

pub(crate) fn decode (opcode: u16) -> Instruction {
    let nibble  = extract_nibbles(opcode, 3, 1);
    let x       = extract_nibbles(opcode, 2, 1) as usize;
    let y       = extract_nibbles(opcode, 1, 1) as usize;
    let n       = extract_nibbles(opcode, 0, 1) as u8;
    let nn      = extract_nibbles(opcode, 0, 2) as u8;
    let nnn     = extract_nibbles(opcode, 0, 3);

    match nibble {
        0x0 => match y {
            0xE => Instruction::Cls,
            _ => Instruction::None
        }
        0x1 => Instruction::Jmp(nnn),
        0x6 => Instruction::Mov(x, nn),
        0x7 => Instruction::Add(x, nn),
        0xA => Instruction::Mvi(nnn),
        0xD => Instruction::Sprite(x, y, n),
        _ => Instruction::None
    }
}