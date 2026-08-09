pub(crate) struct Display {
    content: [[bool; Self::WIDTH]; Self::HEIGHT],
}

impl Display {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

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
}