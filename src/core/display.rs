struct Display {
    content: [[bool; Self::WIDTH]; Self::HEIGHT],
}

impl Display {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    fn within_range(&self, row: usize, col: usize) -> bool {
        col < Self::WIDTH && row < Self::HEIGHT
    }

    fn set_pixel(&mut self, row: usize, col: usize, value: bool) -> Result<(), String> {
        if self.within_range(row, col) {
            self.content[row][col] = value;
            Ok(())
        } else {
            Err("Pixel position out of bounds!".to_string())
        }
    }

    fn clear(&mut self) {
    for row in self.content.iter_mut() {
        for pixel in row.iter_mut() {
            *pixel = false;
        }
    }
}
}