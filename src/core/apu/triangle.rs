use super::{length_counter::LengthCounter, linear_counter::LinearCounter, timer::Timer};

const DUTY_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

#[derive(Default)]
pub struct Triangle {
    pub duty_timer: usize,
    pub enabled: bool,
    pub timer: Timer,
    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
}

impl Triangle {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }

    pub fn tick(&mut self) {
        if self.timer.tick()
            && self.enabled
            && !self.length_counter.mute()
            && !self.linear_counter.mute()
        {
            self.duty_timer = (self.duty_timer + 1) % DUTY_TABLE.len();
        }
    }

    pub fn get_sample(&self) -> f32 {
        if self.timer.get_period() < 2 {
            return 7.0;
        }

        DUTY_TABLE[self.duty_timer] as f32
    }
}
