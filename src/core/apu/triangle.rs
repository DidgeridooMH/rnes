use sdl2::audio::AudioCallback;

use super::{length_counter::LengthCounter, linear_counter::LinearCounter, MASTER_VOLUME};

#[derive(Default)]
pub struct Triangle {
    spec_freq: f32,
    pub phase: f32,
    volume: f32,

    pub enabled: bool,

    pub timer: u16,

    pub length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
}

impl Triangle {
    pub fn new(spec_freq: f32) -> Self {
        Self {
            spec_freq,
            volume: 1.8,
            ..Default::default()
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }
}

fn triangle_wave(x: f32) -> f32 {
    if x < 0.25 {
        f32::abs(2.0 * x)
    } else if x < 0.75 {
        1.0 - f32::abs(2.0 * x)
    } else {
        f32::abs(1.0 - 2.0 * x) - 1.0
    }
}

impl AudioCallback for Triangle {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let frequency = 1789773.0 / (32.0 * (self.timer + 1) as f32);
        let phase_inc = frequency / self.spec_freq;
        for (index, x) in out.iter_mut().enumerate() {
            if self.enabled && !self.length_counter.mute() && !self.linear_counter.mute() {
                let wave = triangle_wave(self.phase);
                *x = wave * self.volume * MASTER_VOLUME;
                if index % 2 == 0 {
                    self.phase = (self.phase + phase_inc) % 1.0;
                }
            } else {
                *x = 0.0;
            }
        }
    }
}
