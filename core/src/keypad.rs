pub struct Keypad {
    keys: [bool; Self::NUM_KEYS],
}

impl Keypad {
    const NUM_KEYS: usize = 16;

    pub(crate) fn new() -> Self {
        Keypad { keys: [false; Self::NUM_KEYS] }
    }

    pub(crate) fn is_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub(crate) fn is_pressed_any(&self) -> (usize, bool) {
        for (key, pressed) in self.keys.iter().enumerate() {
            if *pressed {
               return (key, true);
            }
        }
        (0xFF, false)
    }

    pub(crate) fn press_key(&mut self, key: usize) {
        self.keys[key] = true;
    }

    pub(crate) fn release_key(&mut self, key: usize) {
        self.keys[key] = false;
    }
}