use super::{envelope::Envelope, sweep::Sweep, MASTER_VOLUME};
use sdl2::audio::AudioCallback;

const DUTY_CYCLES: [[u32; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

#[derive(Default)]
pub struct Pulse {
    spec_freq: f32,
    pub phase: f32,
    volume: f32,

    pub enabled: bool,

    pub length_counter: u8,
    pub length_counter_halt: bool,
    pub timer: u16,
    pub duty_cycle: u8,

    pub sweep: Sweep,
    pub envelope: Envelope,
}

impl Pulse {
    pub fn new(spec_freq: f32, phase: f32, channel: u8) -> Self {
        Self {
            spec_freq,
            phase,
            volume: 1.0,
            sweep: Sweep::new(channel),
            ..Default::default()
        }
    }

    pub fn step_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}

impl AudioCallback for Pulse {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let frequency = 1789773.0 / (16.0 * (self.timer + 1) as f32);
        let phase_inc = frequency / self.spec_freq;
        for x in out.iter_mut() {
            if self.enabled && self.length_counter > 0 && self.timer >= 8 && !self.sweep.mute() {
                let wave = ((DUTY_CYCLES[self.duty_cycle as usize][(self.phase * 8.0) as usize]
                    as f32)
                    * 2.0)
                    - 1.0;
                let decay = self.envelope.volume() as f32 / 15.0;
                *x = decay * wave * self.volume * MASTER_VOLUME;
                self.phase = (self.phase + phase_inc) % 1.0;
            } else {
                *x = 0.0;
            }
        }
    }
}
