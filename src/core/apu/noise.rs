use sdl2::audio::AudioCallback;

use super::{envelope::Envelope, length_counter::LengthCounter, MASTER_VOLUME};

const NOISE_PERIOD: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    spec_freq: f32,
    pub phase: f32,
    pub envelope: Envelope,
    pub length_counter: LengthCounter,

    pub enabled: bool,
    volume: f32,

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
    pub fn new(spec_freq: f32) -> Self {
        Self {
            phase: 0.0,
            spec_freq,
            shift: 1,
            envelope: Envelope::default(),
            length_counter: LengthCounter::default(),
            enabled: false,
            volume: 1.0,
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

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }

    pub fn set_period(&mut self, lut_index: u8) {
        self.timer = NOISE_PERIOD[lut_index as usize];
    }
}

impl AudioCallback for Noise {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if self.timer > 0 {
            let frequency = 1789773.0 / (93.0 * self.timer as f32);
            let phase_inc = frequency / self.spec_freq;

            let mut shift_value = self.shift;
            for (index, x) in out.iter_mut().enumerate() {
                if self.enabled && !self.length_counter.mute() && self.timer >= 8 {
                    let noise = if shift_value & 1 == 0 { 1.0 } else { -1.0 };
                    let decay = self.envelope.volume() as f32 / 15.0;
                    *x = noise * decay * self.volume * MASTER_VOLUME;
                } else {
                    *x = 0.0;
                }
                if index % 2 == 0 {
                    self.phase += phase_inc;
                    if self.phase >= 1.0 {
                        shift_value = shift(shift_value);
                        self.phase %= 1.0;
                    }
                }
            }
        } else {
            out.fill(0.0);
        }
    }
}
