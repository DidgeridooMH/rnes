#[derive(Default)]
pub struct Envelope {
    start: bool,
    divider: u8,
    decay: u8,
    pub should_loop: bool,
    pub constant_volume: bool,
    pub envelope: u8,
}

impl Envelope {
    pub fn step(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.envelope;
        } else if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = self.envelope;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.should_loop {
                self.decay = 15;
            }
        }
    }

    pub fn reload(&mut self) {
        self.start = true;
    }

    pub fn volume(&self) -> u8 {
        if self.constant_volume {
            self.envelope
        } else {
            self.decay
        }
    }
}
