mod pulse;
use pulse::Pulse;
mod envelope;
mod sweep;
use sweep::SweepSetup;
mod length_counter;
mod triangle;
use triangle::Triangle;
mod linear_counter;
mod noise;
use noise::Noise;
mod dmc;
use dmc::Dmc;
mod timer;

use crate::{audio::AudioOutput, core::Addressable};

const FRAME_COUNTER_FREQ: usize = 1789773 / 240;

pub struct APU {
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    cycle: usize,
    frame_counter: u8,
    sequencer_mode: bool,
    interrupt_inhibit: bool,
    half_frame_flag: bool,
    audio_output: AudioOutput,
    sequence_reset: bool,
}

impl Default for APU {
    fn default() -> Self {
        Self {
            pulse: [Pulse::new(0), Pulse::new(1)],
            triangle: Triangle::default(),
            noise: Noise::new(),
            dmc: Dmc::default(),
            frame_counter: 0,
            cycle: 0,
            sequencer_mode: false,
            interrupt_inhibit: false,
            half_frame_flag: false,
            audio_output: AudioOutput::new(),
            sequence_reset: false,
        }
    }
}

impl APU {
    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.cycle += 1;
            if self.cycle % 2 == 0 {
                self.pulse[0].tick();
                self.pulse[1].tick();
                self.noise.tick();
                self.dmc.tick();
            }
            self.triangle.tick();

            if self.cycle % FRAME_COUNTER_FREQ == 0 {
                self.step_frame_counter();

                if self.half_frame_flag {
                    for pulse in &mut self.pulse {
                        if let Some(new_timer) = pulse.sweep.step() {
                            pulse.timer.set_period(new_timer);
                        }
                    }
                }
                self.half_frame_flag = !self.half_frame_flag;
            }

            let p0_sample = self.pulse[0].get_sample() * 0.0;
            let p1_sample = self.pulse[1].get_sample() * 0.0;
            let pulse_sample = if p0_sample + p1_sample > 0.0 {
                95.88 / ((8128.0 / (p0_sample + p1_sample)) + 100.0)
            } else {
                0.0
            };

            let t_sample = self.triangle.get_sample();
            let n_sample = self.noise.get_sample() * 0.0;
            let tn_mix = (t_sample / 8227.0) + (n_sample / 12241.0);
            let tnd_sample = if tn_mix > 0.0 {
                159.79 / ((1.0 / tn_mix) + 100.0)
            } else {
                0.0
            };

            self.audio_output.push_sample(pulse_sample + tnd_sample);
        }
    }

    fn step_frame_counter(&mut self) {
        if !self.sequencer_mode
            || self.frame_counter != 3
            || (self.sequence_reset && self.sequencer_mode)
        {
            self.pulse[0].envelope.step();
            self.pulse[1].envelope.step();
            self.triangle.linear_counter.step();
            self.noise.envelope.step();
        }

        if self.frame_counter == 1
            || (self.sequencer_mode && self.frame_counter == 4)
            || (!self.sequencer_mode && self.frame_counter == 3)
            || (self.sequence_reset && self.sequencer_mode)
        {
            self.pulse[0].length_counter.step();
            self.pulse[1].length_counter.step();

            self.triangle.length_counter.step();

            self.noise.length_counter.step();
        }

        self.sequence_reset = false;

        self.frame_counter = if self.sequencer_mode {
            (self.frame_counter + 1) % 5
        } else {
            (self.frame_counter + 1) % 4
        };
    }
}

impl Addressable for APU {
    fn read_byte(&mut self, address: u16) -> u8 {
        if address == 0x4015 {
            let mut status = 0u8;
            if !self.pulse[0].length_counter.mute() {
                status |= 1;
            }
            if !self.pulse[1].length_counter.mute() {
                status |= 2;
            }
            if !self.triangle.length_counter.mute() {
                status |= 4;
            }
            if !self.noise.length_counter.mute() {
                status |= 8;
            }
            return status;
        }
        0
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x4000 | 0x4004 => {
                let pulse = if address == 0x4000 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };
                pulse.length_counter.halt = data & 0x20 > 0;
                pulse.duty_cycle = data >> 6;

                pulse.envelope.should_loop = data & 0x20 > 0;
                pulse.envelope.constant_volume = data & 0x10 > 0;
                pulse.envelope.envelope = data & 0xF;
                pulse.envelope.reload();
            }
            0x4001 | 0x4005 => {
                let pulse = if address == 0x4001 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };
                pulse.sweep.setup(SweepSetup(data));
                let timer = pulse.timer.get_period();
                pulse.sweep.reset_target(timer);
            }
            0x4002 | 0x4006 => {
                let pulse = if address == 0x4002 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };

                let new_timer_period = (pulse.timer.get_period() & 0xFF00) | data as u16;
                pulse.timer.set_period(new_timer_period);
                pulse.sweep.reset_target(new_timer_period);
            }
            0x4003 | 0x4007 => {
                let pulse = if address == 0x4003 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };
                pulse.length_counter.set_counter(data >> 3);
                let new_timer_period =
                    (pulse.timer.get_period() & 0xFF) | ((data & 0b111) as u16) << 8;
                pulse.timer.set_period(new_timer_period);
                self.frame_counter = 0;
                pulse.duty_timer = 0;

                pulse.envelope.reload();

                pulse.sweep.reset_target(new_timer_period);
            }
            0x4008 => {
                self.triangle.length_counter.halt = data & 0x80 > 0;
                self.triangle.linear_counter.control = data & 0x80 > 0;
                self.triangle.linear_counter.reload(data & 0x7F);
            }
            0x400A => {
                self.triangle
                    .timer
                    .set_period((self.triangle.timer.get_period() & 0xFF00) | data as u16);
            }
            0x400B => {
                self.triangle.timer.set_period(
                    (self.triangle.timer.get_period() & 0xFF) | (((data & 0b111) as u16) << 8),
                );
                self.triangle.length_counter.set_counter(data >> 3);
                self.triangle.linear_counter.reload_current();
            }
            0x400C => {
                self.noise.length_counter.halt = data & 0x20 > 0;

                self.noise.envelope.should_loop = data & 0x20 > 0;
                self.noise.envelope.constant_volume = data & 0x10 > 0;
                self.noise.envelope.envelope = data & 0xF;
                self.noise.envelope.reload();
            }
            0x400E => {
                self.noise.set_period(data & 0xF);
            }
            0x400F => {
                self.noise.length_counter.set_counter(data >> 3);
                self.noise.envelope.reload();
            }
            0x4015 => {
                self.pulse[0].set_enabled(data & 1 > 0);
                self.pulse[1].set_enabled(data & 2 > 0);
                self.triangle.set_enabled(data & 4 > 0);
                self.noise.set_enabled(data & 8 > 0);
            }
            0x4017 => {
                self.sequencer_mode = data & 0x80 > 0;
                self.interrupt_inhibit = data & 0x40 > 0;
                self.frame_counter = 0;
                self.sequence_reset = true;
            }
            _ => {}
        }
    }
}
