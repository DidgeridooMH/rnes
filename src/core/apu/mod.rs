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

use std::time::{Duration, Instant};

use crate::core::Addressable;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

const FRAME_COUNTER_FREQ: usize = 1789773 / 240;
const SAMPLE_BUFFER_LENGTH: usize = 735 * 2;

pub struct APU {
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    sound_sampler: AudioQueue<f32>,
    cycle: usize,
    frame_counter: u8,
    sequencer_mode: bool,
    last_frame: Instant,
    interrupt_inhibit: bool,
    half_frame_flag: bool,
    sample_buffer: [f32; SAMPLE_BUFFER_LENGTH],
    sample_cursor: usize,
    sample_starvation: usize,
    sample_starvation_change: bool,
}

impl Default for APU {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let sound_sampler = audio_subsystem.open_queue(None, &desired_spec).unwrap();
        sound_sampler.resume();

        Self {
            pulse: [Pulse::new(0), Pulse::new(1)],
            triangle: Triangle::default(),
            noise: Noise::new(),
            sound_sampler,
            frame_counter: 0,
            cycle: 0,
            sequencer_mode: false,
            last_frame: Instant::now(),
            interrupt_inhibit: false,
            half_frame_flag: false,
            sample_buffer: [0.0; SAMPLE_BUFFER_LENGTH],
            sample_cursor: 0,
            sample_starvation: 0,
            sample_starvation_change: false,
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
            }
            self.triangle.tick();

            if self.cycle % 40 == 0 {
                if self.sound_sampler.size() == 0 {
                    self.sample_starvation += 1;
                    self.sample_starvation_change = true;
                }
                let p0_sample = self.pulse[0].get_sample();
                let p1_sample = self.pulse[1].get_sample();
                let pulse_sample = 95.88 / ((8128.0 / (p0_sample + p1_sample)) + 100.0);

                let t_sample = self.triangle.get_sample();
                let n_sample = self.noise.get_sample();
                let tnd_sample =
                    159.79 / ((1.0 / ((t_sample / 8227.0) + (n_sample / 12241.0))) + 100.0);

                if self.sound_sampler.size() / (SAMPLE_BUFFER_LENGTH as u32)
                    < (0.5 / (SAMPLE_BUFFER_LENGTH as f32 / 44100.0)).ceil() as u32
                {
                    self.sample_buffer[self.sample_cursor] = (pulse_sample + tnd_sample) * 3.0;
                    self.sample_cursor += 1;
                }
                if self.sample_cursor == self.sample_buffer.len() {
                    self.sound_sampler.queue_audio(&self.sample_buffer).unwrap();
                    self.sample_cursor = 0;
                }
            }

            if self.cycle % FRAME_COUNTER_FREQ == 0 {
                self.step_frame_counter();

                if self.half_frame_flag {
                    for pulse in &mut self.pulse {
                        if let Some(new_timer) = pulse.sweep.step() {
                            pulse.timer_reload = new_timer;
                            pulse.timer = new_timer;
                        }
                    }
                }
                self.half_frame_flag = !self.half_frame_flag;
            }
        }
    }

    fn step_frame_counter(&mut self) {
        while self.last_frame.elapsed() < Duration::from_micros(16666 / 4) {}
        self.last_frame = Instant::now();

        if !self.sequencer_mode || self.frame_counter != 3 {
            self.pulse[0].envelope.step();
            self.pulse[1].envelope.step();
            self.triangle.linear_counter.step();
            self.noise.envelope.step();
        }

        if self.frame_counter == 1
            || (self.sequencer_mode && self.frame_counter == 4)
            || (!self.sequencer_mode && self.frame_counter == 3)
        {
            self.pulse[0].length_counter.step();
            self.pulse[1].length_counter.step();

            self.triangle.length_counter.step();

            self.noise.length_counter.step();
        }

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
                let mut pulse = if address == 0x4000 {
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
                let timer = pulse.timer_reload;
                pulse.sweep.reset_target(timer);
            }
            0x4002 | 0x4006 => {
                let mut pulse = if address == 0x4002 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };
                pulse.timer_reload = (pulse.timer_reload & 0xFF00) | data as u16;
                pulse.timer = pulse.timer_reload;
                let timer = pulse.timer_reload;
                pulse.sweep.reset_target(timer);
            }
            0x4003 | 0x4007 => {
                let mut pulse = if address == 0x4003 {
                    &mut self.pulse[0]
                } else {
                    &mut self.pulse[1]
                };
                pulse.length_counter.set_counter(data >> 3);
                pulse.timer_reload = (pulse.timer_reload & 0xFF) | ((data & 0b111) as u16) << 8;
                pulse.timer = pulse.timer_reload;
                self.frame_counter = 0;
                pulse.duty_timer = 0;

                pulse.envelope.reload();

                let timer = pulse.timer_reload;
                pulse.sweep.reset_target(timer);
            }
            0x4008 => {
                self.triangle.length_counter.halt = data & 0x80 > 0;
                self.triangle.linear_counter.control = data & 0x80 > 0;
                self.triangle.linear_counter.reload(data & 0x7F);
            }
            0x400A => {
                self.triangle.timer_reload = (self.triangle.timer_reload & 0xFF00) | data as u16;
                self.triangle.timer = self.triangle.timer_reload;
                self.triangle.linear_counter.reload_current();
            }
            0x400B => {
                self.triangle.timer_reload =
                    (self.triangle.timer_reload & 0xFF) | ((data & 0b111) as u16) << 8;
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
            }
            _ => {}
        }
    }
}
