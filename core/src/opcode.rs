pub(crate) enum Instruction {
    Unknown,
    Cls,                        //00E0
    Rts,                        //00EE
    Low,                        //00FE
    High,                       //00FF
    Jmp(usize),                 //1nnn
    Jsr(usize),                 //2nnn
    SkeqConst(usize, u8),       //3xnn
    SkneConst(usize, u8),       //4xnn
    Skeq(usize, usize),         //5xy0
    MovConst(usize, u8),        //6xnn
    AddConst(usize, u8),        //7xnn
    Mov(usize, usize),          //8xy0
    Or(usize, usize),           //8xy1
    And(usize, usize),          //8xy2
    Xor(usize, usize),          //8xy3
    Add(usize, usize),          //8xy4
    Sub(usize, usize),          //8xy5
    Shr(usize, usize),          //8xy6 (8x06)
    Rsb(usize, usize),          //8xy7
    Shl(usize, usize),          //8xyE (8x0E)
    Skne(usize, usize),         //9xy0
    Mvi(usize),                 //Annn
    Jmi(usize),                 //Bnnn
    Rand(usize, u8),            //Cxnn
    Sprite(usize, usize, u8),   //Dxyn
    Skpr(usize),                //Ex9E
    Skup(usize),                //ExA1
    Gdelay(usize),              //Fx07
    Key(usize),                 //Fx0A
    Sdelay(usize),              //Fx15
    Ssound(usize),              //Fx18
    Adi(usize),                 //Fx1E
    Font(usize),                //Fx29
    Xfont(usize),               //Fx30
    Bcd(usize),                 //Fx33
    Str(usize),                 //Fx55
    Ldr(usize),                 //Fx65
}

/// Extracts `length` nibbles from `opcode`, starting `position` nibbles from the right (LSB).
///
/// e.g. `extract_nibbles(0xABCD, 2, 1)` returns `0xB` (the third nibble from the right).
pub(crate) fn extract_nibbles(opcode: u16, position: u8, length: u8) -> u16 {
    (opcode >> (4 * position)) & ((1 << (4 * length)) - 1)
}

pub(crate) fn decode (opcode: u16) -> Instruction {
    let nibble  = extract_nibbles(opcode, 3, 1);
    let x       = extract_nibbles(opcode, 2, 1) as usize;
    let y       = extract_nibbles(opcode, 1, 1) as usize;
    let n       = extract_nibbles(opcode, 0, 1) as u8;
    let nn      = extract_nibbles(opcode, 0, 2) as u8;
    let nnn     = extract_nibbles(opcode, 0, 3) as usize;

    match nibble {
        0x0 => match nn {
            0xE0 => Instruction::Cls,
            0xEE => Instruction::Rts,
            0xFE => Instruction::Low,
            0xFF => Instruction::High,

            _ => Instruction::Unknown
        }
        0x1 => Instruction::Jmp(nnn),
        0x2 => Instruction::Jsr(nnn),
        0x3 => Instruction::SkeqConst(x, nn),
        0x4 => Instruction::SkneConst(x, nn),
        0x5 => Instruction::Skeq(x, y),  
        0x6 => Instruction::MovConst(x, nn),
        0x7 => Instruction::AddConst(x, nn),
        0x8 => match n {
            0x0 => Instruction::Mov(x, y),
            0x1 => Instruction::Or(x, y),
            0x2 => Instruction::And(x, y),
            0x3 => Instruction::Xor(x, y),
            0x4 => Instruction::Add(x, y),
            0x5 => Instruction::Sub(x, y),
            0x6 => Instruction::Shr(x, y),
            0x7 => Instruction::Rsb(x, y),
            0xE => Instruction::Shl(x, y),

            _ => Instruction::Unknown
        }
        0x9 => Instruction::Skne(x, y),
        0xA => Instruction::Mvi(nnn),
        0xB => Instruction::Jmi(nnn),
        0xC => Instruction::Rand(x, nn),
        0xD => Instruction::Sprite(x, y, n),
        0xE => match n {
            0x1 => Instruction::Skup(x),
            0xE => Instruction::Skpr(x),

            _ => Instruction::Unknown
        }
        0xF => match nn {
            0x07 => Instruction::Gdelay(x),
            0x0A => Instruction::Key(x),
            0x15 => Instruction::Sdelay(x),
            0x18 => Instruction::Ssound(x),
            0x1E => Instruction::Adi(x),
            0x29 => Instruction::Font(x),
            0x30 => Instruction::Xfont(x),
            0x33 => Instruction::Bcd(x),
            0x55 => Instruction::Str(x),
            0x65 => Instruction::Ldr(x),

            _ => Instruction::Unknown
        }

        _ => Instruction::Unknown
    }
}