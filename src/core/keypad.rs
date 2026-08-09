pub struct Keypad {
    keys: [bool; Self::NUM_KEYS],
}

impl Keypad {
    const NUM_KEYS: usize = 16;

    pub(crate) fn is_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }
}