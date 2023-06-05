use super::{length_counter::LengthCounter, linear_counter::LinearCounter};

const DUTY_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

#[derive(Default)]
pub struct Triangle {
    pub duty_timer: usize,
    pub enabled: bool,
    pub timer: u16,
    pub timer_reload: u16,
    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
}

impl Triangle {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }

    pub fn tick(&mut self) {
        self.timer -= 1;
        if self.timer == 0 {
            self.duty_timer = (self.duty_timer + 1) % DUTY_TABLE.len();
            self.timer = self.timer_reload;
        }
    }

    pub fn get_sample(&self) -> f32 {
        if self.enabled && !self.length_counter.mute() && !self.linear_counter.mute() {
            DUTY_TABLE[self.duty_timer] as f32 / 15.0
        } else {
            0.0
        }
    }
}
