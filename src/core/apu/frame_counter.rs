#[derive(Default)]
pub struct FrameCounter {
    counter: u32,
    extended_step: bool,
}

impl FrameCounter {
    pub fn step(&mut self) {
        self.counter += 1;
        if self.extended_step {
            self.counter %= 5;
        } else {
            self.counter %= 4;
        }
    }

    pub fn set_mode(&mut self, extended_step: bool) {
        self.extended_step = extended_step;
        self.counter = 0;
    }

    pub fn half_clock(&self) -> bool {
        if self.extended_step {
            self.counter == 0 || self.counter == 2
        } else {
            self.counter == 1 || self.counter == 3
        }
    }

    pub fn quarter_clock(&self) -> bool {
        self.counter != 4
    }
}
