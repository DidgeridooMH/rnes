const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Default)]
pub struct LengthCounter {
    enabled: bool,
    counter: u8,
    pub halt: bool,
}

impl LengthCounter {
    pub fn step(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }

    pub fn mute(&self) -> bool {
        self.counter == 0
    }

    pub fn set_counter(&mut self, new_counter: u8) {
        if self.enabled {
            self.counter = LENGTH_TABLE[new_counter as usize];
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.counter = 0;
        }
    }
}
