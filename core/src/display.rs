use crate::{error::Chip8Error, utils::extract_bit};

/// A 64 x 32 monochrome CHIP-8 display buffer.
pub struct Display {
    content: [[bool; Self::WIDTH]; Self::HEIGHT],
    dirty: bool,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    /// Display width in pixels.
    pub const WIDTH: usize = 64;
    /// Display height in pixels.
    pub const HEIGHT: usize = 32;

    pub(crate) fn new() -> Self {
        Display {
            content: [[false; Self::WIDTH]; Self::HEIGHT],
            dirty: false,
        }
    }

    /// Return the current framebuffer.
    pub fn get_content(&self) -> &[[bool; Self::WIDTH]; Self::HEIGHT] {
        &self.content
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false)
    }

    /// Set one pixel without applying sprite XOR behavior.
    pub(crate) fn set_pixel(
        &mut self,
        row: usize,
        col: usize,
        value: bool,
    ) -> Result<(), Chip8Error> {
        if self.out_of_bounds(row, col) {
            Err(Chip8Error::DisplayOutOfBounds { row, col })
        } else {
            self.content[row][col] = value;
            Ok(())
        }
    }

    /// Clear every pixel in the framebuffer.
    pub(crate) fn clear(&mut self) {
        for row in self.content.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }

    /// Draw a sprite using XOR semantics and return whether a collision occurred.
    pub(crate) fn draw_sprite(
        &mut self,
        x_start: usize,
        y_start: usize,
        sprite: &[u8],
        wrap: bool,
    ) -> Result<bool, Chip8Error> {
        if self.out_of_bounds(y_start, x_start) {
            return Err(Chip8Error::DisplayOutOfBounds {
                row: (y_start),
                col: (x_start),
            });
        }

        let mut collision = false;
        'row: for (row, &byte) in sprite.iter().enumerate() {
            'column: for bit in 0..8 {
                let value = extract_bit(byte, 7 - bit as u8);

                if value == 1 {
                    let mut x = x_start + bit;
                    if x >= Self::WIDTH {
                        if wrap {
                            x -= Self::WIDTH;
                        } else {
                            break 'column;
                        }
                    }

                    let mut y = y_start + row;
                    if y >= Self::HEIGHT {
                        if wrap {
                            y -= Self::HEIGHT;
                        } else {
                            break 'row;
                        }
                    }

                    match self.content[y][x] {
                        false => self.content[y][x] = true,
                        true => {
                            self.content[y][x] = false;
                            collision = true;
                        }
                    }
                    self.dirty = true;
                }
            }
        }

        Ok(collision)
    }

    /// Check whether a display coordinate is outside the framebuffer.
    fn out_of_bounds(&self, row: usize, col: usize) -> bool {
        row >= Self::HEIGHT || col >= Self::WIDTH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_display_is_empty() {
        let display = Display::new();

        assert_eq!(
            display.get_content(),
            &[[false; Display::WIDTH]; Display::HEIGHT]
        );
    }

    #[test]
    fn test_set_pixel() {
        let mut display = Display::new();

        assert_eq!(display.set_pixel(3, 5, true), Ok(()));
        assert!(display.get_content()[3][5]);

        assert_eq!(display.set_pixel(3, 5, false), Ok(()));
        assert!(!display.get_content()[3][5]);
    }

    #[test]
    fn test_set_pixel_out_of_bounds() {
        let mut display = Display::new();

        assert_eq!(
            display.set_pixel(Display::HEIGHT, 0, true),
            Err(Chip8Error::DisplayOutOfBounds {
                row: Display::HEIGHT,
                col: 0
            })
        );
        assert_eq!(
            display.set_pixel(0, Display::WIDTH, true),
            Err(Chip8Error::DisplayOutOfBounds {
                row: 0,
                col: Display::WIDTH
            })
        );
    }

    #[test]
    fn test_clear() {
        let mut display = Display::new();
        display.set_pixel(0, 0, true).unwrap();
        display
            .set_pixel(Display::HEIGHT - 1, Display::WIDTH - 1, true)
            .unwrap();

        display.clear();

        assert_eq!(
            display.get_content(),
            &[[false; Display::WIDTH]; Display::HEIGHT]
        );
    }

    #[test]
    fn test_draw_sprite() {
        let mut display = Display::new();

        assert_eq!(display.draw_sprite(2, 3, &[0b1010_0001], false), Ok(false));

        assert!(display.get_content()[3][2]);
        assert!(display.get_content()[3][4]);
        assert!(display.get_content()[3][9]);
        assert!(!display.get_content()[3][3]);
    }

    #[test]
    fn test_draw_sprite_collision() {
        let mut display = Display::new();
        display.set_pixel(3, 2, true).unwrap();
        display.set_pixel(3, 4, true).unwrap();

        assert_eq!(display.draw_sprite(2, 3, &[0b1010_0001], false), Ok(true));

        assert!(!display.get_content()[3][2]);
        assert!(!display.get_content()[3][4]);
    }

    #[test]
    fn test_draw_sprite_clips_at_display_edges() {
        let mut display = Display::new();

        assert_eq!(
            display.draw_sprite(
                Display::WIDTH - 2,
                Display::HEIGHT - 1,
                &[0b1111_1111, 0b1111_1111],
                false,
            ),
            Ok(false)
        );

        assert!(display.get_content()[Display::HEIGHT - 1][Display::WIDTH - 2]);
        assert!(display.get_content()[Display::HEIGHT - 1][Display::WIDTH - 1]);
        assert!(!display.get_content()[Display::HEIGHT - 2][Display::WIDTH - 1]);
    }

    #[test]
    fn test_draw_sprite_wraps_at_display_edges() {
        let mut display = Display::new();

        assert_eq!(
            display.draw_sprite(
                Display::WIDTH - 2,
                Display::HEIGHT - 1,
                &[0b1111_1111, 0b1000_0000],
                true,
            ),
            Ok(false)
        );

        assert!(display.get_content()[Display::HEIGHT - 1][Display::WIDTH - 2]);
        assert!(display.get_content()[Display::HEIGHT - 1][Display::WIDTH - 1]);
        assert!(display.get_content()[Display::HEIGHT - 1][0]);
        assert!(display.get_content()[Display::HEIGHT - 1][1]);
        assert!(display.get_content()[Display::HEIGHT - 1][2]);
        assert!(display.get_content()[Display::HEIGHT - 1][3]);
        assert!(display.get_content()[Display::HEIGHT - 1][4]);
        assert!(display.get_content()[Display::HEIGHT - 1][5]);
        assert!(display.get_content()[0][Display::WIDTH - 2]);
    }

    #[test]
    fn test_draw_sprite_out_of_bounds() {
        let mut display = Display::new();

        assert_eq!(
            display.draw_sprite(Display::WIDTH, 0, &[0], false),
            Err(Chip8Error::DisplayOutOfBounds {
                row: 0,
                col: Display::WIDTH
            })
        );
        assert_eq!(
            display.draw_sprite(0, Display::HEIGHT, &[0], false),
            Err(Chip8Error::DisplayOutOfBounds {
                row: Display::HEIGHT,
                col: 0
            })
        );
    }

    #[test]
    fn test_out_of_bounds() {
        let display = Display::new();

        assert!(!display.out_of_bounds(0, 0));
        assert!(!display.out_of_bounds(Display::HEIGHT - 1, Display::WIDTH - 1));
        assert!(display.out_of_bounds(Display::HEIGHT, 0));
        assert!(display.out_of_bounds(0, Display::WIDTH));
    }
}
