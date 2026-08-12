pub struct Keypad {
    keys: [bool; Self::NUM_KEYS],
}

impl Keypad {
    const NUM_KEYS: usize = 16;

    pub(crate) fn is_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub(crate) fn press_key(&mut self, key: usize) {
        self.keys[key] = true;
    }

    pub(crate) fn release_key(&mut self, key: usize) {
        self.keys[key] = false;
    }
}