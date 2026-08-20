use crate::error::Chip8Error;

pub(crate) struct Keypad {
    keys: [bool; Self::NUM_KEYS],
}

impl Default for Keypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Keypad {
    /// Number of keys in the CHIP-8 keypad.
    const NUM_KEYS: usize = 16;

    /// Create a keypad with every key released.
    pub(crate) fn new() -> Self {
        Keypad {
            keys: [false; Self::NUM_KEYS],
        }
    }

    /// Check a key state, returning an error for values outside `0..16`.
    pub(crate) fn is_pressed(&self, key: usize) -> Result<bool, Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            Ok(self.keys[key])
        }
    }

    /// Return the lowest-numbered pressed key, or `(0xFF, false)` if none is pressed.
    pub(crate) fn is_pressed_any(&self) -> (usize, bool) {
        for (key, pressed) in self.keys.iter().enumerate() {
            if *pressed {
                return (key, true);
            }
        }
        (0xFF, false)
    }

    /// Mark a key as pressed.
    pub(crate) fn press_key(&mut self, key: usize) -> Result<(), Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            self.keys[key] = true;
            Ok(())
        }
    }

    /// Mark a key as released.
    pub(crate) fn release_key(&mut self, key: usize) -> Result<(), Chip8Error> {
        if self.out_of_bounds(key) {
            Err(Chip8Error::KeypadOutOfBounds { key })
        } else {
            self.keys[key] = false;
            Ok(())
        }
    }

    /// Check whether a key index is outside the keypad range.
    fn out_of_bounds(&self, key: usize) -> bool {
        key >= Self::NUM_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_keypad_has_no_pressed_keys() {
        let keypad = Keypad::new();

        for key in 0..Keypad::NUM_KEYS {
            assert_eq!(keypad.is_pressed(key), Ok(false));
        }
        assert_eq!(keypad.is_pressed_any(), (0xFF, false));
    }

    #[test]
    fn test_press_and_release_each_valid_key() {
        let mut keypad = Keypad::new();

        for key in 0..Keypad::NUM_KEYS {
            assert_eq!(keypad.press_key(key), Ok(()));
            assert_eq!(keypad.is_pressed(key), Ok(true));
            assert_eq!(keypad.release_key(key), Ok(()));
            assert_eq!(keypad.is_pressed(key), Ok(false));
        }
    }

    #[test]
    fn test_invalid_key_returns_error() {
        let mut keypad = Keypad::new();
        let invalid_key = Keypad::NUM_KEYS;
        let expected_error = Chip8Error::KeypadOutOfBounds { key: invalid_key };

        assert_eq!(keypad.is_pressed(invalid_key), Err(expected_error));
        assert_eq!(keypad.press_key(invalid_key), Err(expected_error));
        assert_eq!(keypad.release_key(invalid_key), Err(expected_error));
    }

    #[test]
    fn test_is_pressed_any_returns_lowest_pressed_key() {
        let mut keypad = Keypad::new();

        keypad.press_key(0xA).unwrap();
        keypad.press_key(0x3).unwrap();

        assert_eq!(keypad.is_pressed_any(), (0x3, true));

        keypad.release_key(0x3).unwrap();
        assert_eq!(keypad.is_pressed_any(), (0xA, true));
    }

    #[test]
    fn test_out_of_bounds() {
        let keypad = Keypad::new();

        assert!(!keypad.out_of_bounds(0));
        assert!(!keypad.out_of_bounds(Keypad::NUM_KEYS - 1));
        assert!(keypad.out_of_bounds(Keypad::NUM_KEYS));
    }
}
