use super::{envelope::Envelope, length_counter::LengthCounter, sweep::Sweep, timer::Timer};

const DUTY_CYCLES: [[u32; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

#[derive(Default)]
pub struct Pulse {
    pub enabled: bool,

    pub timer: Timer,
    pub duty_cycle: u8,
    pub duty_timer: usize,

    pub sweep: Sweep,
    pub envelope: Envelope,
    pub length_counter: LengthCounter,
}

impl Pulse {
    pub fn new(channel: u8) -> Self {
        Self {
            sweep: Sweep::new(channel),
            ..Default::default()
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.set_enabled(enabled);
    }

    pub fn tick(&mut self) {
        if self.timer.tick() {
            self.duty_timer = (self.duty_timer + 1) % 8;
        }
    }

    pub fn get_sample(&self) -> f32 {
        if self.enabled
            && !self.length_counter.mute()
            && self.timer.get_period() >= 8
            && !self.sweep.mute()
        {
            let wave = DUTY_CYCLES[self.duty_cycle as usize][self.duty_timer] as f32;
            let decay = self.envelope.volume() as f32 / 15.0;
            wave * decay
        } else {
            0.0
        }
    }
}
