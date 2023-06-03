use crate::core::MASTER_VOLUME;
use sdl2::audio::AudioCallback;

pub struct Pulse {
    spec_freq: f32,
    phase: f32,
    volume: f32,

    pub enabled: bool,

    pub length_counter: u8,
    pub length_counter_halt: bool,
    pub use_constant_volume: bool,
    pub constant_volume: u8,
    decay: u8,
    pub timer: u16,
}

impl Pulse {
    pub fn new(spec_freq: f32, phase: f32) -> Self {
        Self {
            spec_freq,
            phase,
            volume: 1.0,
            enabled: false,
            length_counter: 0,
            length_counter_halt: false,
            use_constant_volume: false,
            constant_volume: 15,
            decay: 15,
            timer: 0,
        }
    }

    pub fn step(&mut self) {
        if self.use_constant_volume || (self.decay == 0 && self.length_counter_halt) {
            self.reset_envelope();
        } else if self.decay > 0 {
            self.decay -= 1;
        }
    }

    pub fn step_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn reset_envelope(&mut self) {
        self.decay = self.constant_volume;
    }
}

impl AudioCallback for Pulse {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let frequency = 1789773.0 / (16.0 * (self.timer + 1) as f32);
        let phase_inc = frequency / self.spec_freq;
        for x in out.iter_mut() {
            if self.enabled && self.length_counter > 0 && self.timer >= 8 {
                let wave = if self.phase <= 0.5 { 1.0 } else { -1.0 };
                let decay = self.decay as f32 / 15.0;
                *x = decay * wave * self.volume * MASTER_VOLUME;
                self.phase = (self.phase + phase_inc) % 1.0;
            } else {
                *x = 0.0;
            }
        }
    }
}
