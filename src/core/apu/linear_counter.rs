#[derive(Default)]
pub struct LinearCounter {
    reload: bool,
    pub control: bool,
    reload_value: u8,
    value: u8,
}

impl LinearCounter {
    pub fn step(&mut self) {
        if self.reload {
            self.value = self.reload_value;
        } else if self.value > 0 {
            self.value -= 1;
        }

        if !self.control {
            self.reload = false;
        }
    }

    pub fn mute(&self) -> bool {
        self.value == 0
    }

    pub fn reload(&mut self, reload_value: u8) {
        self.reload = true;
        self.reload_value = reload_value;
    }

    pub fn reload_current(&mut self) {
        self.reload = true;
    }
}
