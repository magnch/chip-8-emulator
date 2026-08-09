pub enum Instruction {
    Cls,                        //00E0
    Jmp(u16),                   //1nnn
    Mov(usize, u16),            //6xnn
    Add(usize, u16),            //7xnn
    Mvi(u16),                   //Annn
    Sprite(usize, usize, u8),   //Dxyn
}