use crate::error::Chip8Error;

pub(crate) struct Keypad {
    keys: [bool; Self::NUM_KEYS],
}

impl Keypad {
    const NUM_KEYS: usize = 16;

    pub(crate) fn new() -> Self {
        Keypad { keys: [false; Self::NUM_KEYS] }
    }

    pub(crate) fn is_pressed(&self, key: usize) -> Result<bool, Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            Ok(self.keys[key])
        }
    }

    pub(crate) fn is_pressed_any(&self) -> (usize, bool) {
        for (key, pressed) in self.keys.iter().enumerate() {
            if *pressed {
               return (key, true);
            }
        }
        (0xFF, false)
    }

    pub(crate) fn press_key(&mut self, key: usize) -> Result<(), Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            self.keys[key] = true;
            Ok(())
        }
    }

    pub(crate) fn release_key(&mut self, key: usize) -> Result<(), Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            self.keys[key] = false;
            Ok(())
        }
    }

    fn out_of_bounds(&self, key: usize) -> bool {
        key >= Self::NUM_KEYS
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keypad() {
        let mut keypad = Keypad::new();

        for i in 0..Keypad::NUM_KEYS {
            assert!(!keypad.out_of_bounds(i));
            assert!(!keypad.is_pressed(i).expect(""));
            assert!(!keypad.press_key(i).is_err());
            assert!(keypad.is_pressed(i).expect(""));
            assert!(!keypad.release_key(i).is_err());
            assert!(!keypad.is_pressed(i).expect(""));
        }
        assert!(!keypad.is_pressed_any().1);
        assert!(keypad.out_of_bounds(16));
        assert!(keypad.press_key(16).is_err());
        assert!(!keypad.press_key(0xA).is_err());
        assert_eq!(keypad.is_pressed_any(), (0xA, true));
    }
}