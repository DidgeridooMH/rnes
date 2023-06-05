use super::{envelope::Envelope, length_counter::LengthCounter};

const NOISE_PERIOD: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    pub envelope: Envelope,
    pub length_counter: LengthCounter,
    pub enabled: bool,
    pub timer: u16,
    current_timer: u16,
    pub shift: u16,
}

fn shift(mut value: u16) -> u16 {
    let feedback = (value & 1) ^ ((value >> 1) & 1);
    value >>= 1;
    value | feedback << 14
}

impl Noise {
    pub fn new() -> Self {
        Self {
            shift: 1,
            envelope: Envelope::default(),
            length_counter: LengthCounter::default(),
            enabled: false,
            timer: 0,
            current_timer: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_timer -= 1;
        if self.current_timer == 0 {
            self.shift = shift(self.shift);
            self.current_timer = self.timer;
        }
    }

    pub fn get_sample(&self) -> f32 {
        if self.enabled && !self.length_counter.mute() && self.timer >= 8 && self.shift & 1 == 0 {
            self.envelope.volume() as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }

    pub fn set_period(&mut self, lut_index: u8) {
        self.timer = NOISE_PERIOD[lut_index as usize];
    }
}
