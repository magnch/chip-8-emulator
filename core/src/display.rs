pub struct Display {
    content: [[bool; Self::WIDTH]; Self::HEIGHT],
}

impl Display {
    pub const WIDTH: usize = 64;
    pub const HEIGHT: usize = 32;

    pub(crate) fn new() -> Self {
        Display { content: [[false; Self::WIDTH]; Self::HEIGHT] }
    }

    pub fn get_content(&self) -> &[[bool; Self::WIDTH]; Self::HEIGHT] {
        &self.content
    }

    pub(crate) fn set_pixel(&mut self, row: usize, col: usize, value: bool) {
        self.content[row][col] = value;
    }

    pub(crate) fn clear(&mut self) {
        for row in self.content.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }

    pub(crate) fn draw_sprite(&mut self, x_start: usize, y_start: usize, sprite: &[u8]) -> bool {
        let mut collision = false;

        'row: for (row, &byte) in sprite.iter().enumerate() {
            'column: for bit in 0..8 {
                let value = (byte >> (7 - bit)) & 1;
                if value == 1 {
                    let x = x_start + bit;
                    if x >= Self::WIDTH {
                        break 'column;
                    }
                    let y = y_start + row;
                    if y >= Self::HEIGHT {
                        break 'row;
                    }
                    match self.content[y][x] {
                        false => self.content[y][x] = true,
                        true => {
                            self.content[y][x] = false;
                            collision = true;
                        }
                    }

                }

            }
        }
        collision
    }
}