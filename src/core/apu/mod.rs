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
use sdl2::audio::{AudioDevice, AudioSpecDesired};

pub const MASTER_VOLUME: f32 = 0.05;
const FRAME_COUNTER_FREQ: usize = 1789773 / 240;

pub struct APU {
    pulse: [AudioDevice<Pulse>; 2],
    triangle: AudioDevice<Triangle>,
    noise: AudioDevice<Noise>,
    cycle: usize,
    frame_counter: u8,
    sequencer_mode: bool,
    last_frame: Instant,
    interrupt_inhibit: bool,
    half_frame_flag: bool,
}

impl Default for APU {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(96000),
            channels: Some(2),
            samples: Some(64),
        };

        let pulse_device0 = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                Pulse::new(spec.freq as f32, 0.0, 0)
            })
            .unwrap();
        //pulse_device0.resume();

        let pulse_device1 = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                Pulse::new(spec.freq as f32, 0.5, 1)
            })
            .unwrap();
        //pulse_device1.resume();

        let triangle = audio_subsystem
            .open_playback(None, &desired_spec, |spec| Triangle::new(spec.freq as f32))
            .unwrap();
        //triangle.resume();

        let noise = audio_subsystem
            .open_playback(None, &desired_spec, |spec| Noise::new(spec.freq as f32))
            .unwrap();
        //noise.resume();

        Self {
            pulse: [pulse_device0, pulse_device1],
            triangle,
            noise,
            frame_counter: 0,
            cycle: 0,
            sequencer_mode: false,
            last_frame: Instant::now(),
            interrupt_inhibit: false,
            half_frame_flag: false,
        }
    }
}

impl APU {
    pub fn tick(&mut self, cycles: usize) {
        self.cycle += cycles;

        {
            let mut noise = self.noise.lock();
            for _ in 0..(cycles / 2) {
                noise.tick();
            }
        }

        if self.cycle >= FRAME_COUNTER_FREQ {
            self.step_frame_counter();
            self.cycle %= FRAME_COUNTER_FREQ;

            if self.half_frame_flag {
                for pulse in &mut self.pulse {
                    let mut pulse = pulse.lock();
                    if let Some(new_timer) = pulse.sweep.step() {
                        pulse.timer = new_timer;
                    }
                }
            }
            self.half_frame_flag = !self.half_frame_flag;
        }
    }

    fn step_frame_counter(&mut self) {
        while self.last_frame.elapsed() < Duration::from_micros(16666 / 4) {}
        self.last_frame = Instant::now();

        if !self.sequencer_mode || self.frame_counter != 3 {
            self.pulse[0].lock().envelope.step();
            self.pulse[1].lock().envelope.step();
            self.triangle.lock().linear_counter.step();
            self.noise.lock().envelope.step();
        }

        if self.frame_counter == 1
            || (self.sequencer_mode && self.frame_counter == 4)
            || (!self.sequencer_mode && self.frame_counter == 3)
        {
            self.pulse[0].lock().length_counter.step();
            self.pulse[1].lock().length_counter.step();

            self.triangle.lock().length_counter.step();

            self.noise.lock().length_counter.step();
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
            if !self.pulse[0].lock().length_counter.mute() {
                status |= 1;
            }
            if !self.pulse[1].lock().length_counter.mute() {
                status |= 2;
            }
            if !self.triangle.lock().length_counter.mute() {
                status |= 4;
            }
            if !self.noise.lock().length_counter.mute() {
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
                    self.pulse[0].lock()
                } else {
                    self.pulse[1].lock()
                };
                pulse.length_counter.halt = data & 0x20 > 0;
                pulse.duty_cycle = data >> 6;

                pulse.envelope.should_loop = data & 0x20 > 0;
                pulse.envelope.constant_volume = data & 0x10 > 0;
                pulse.envelope.envelope = data & 0xF;
                pulse.envelope.reload();
            }
            0x4001 | 0x4005 => {
                let mut pulse = if address == 0x4001 {
                    self.pulse[0].lock()
                } else {
                    self.pulse[1].lock()
                };
                pulse.sweep.setup(SweepSetup(data));
                let timer = pulse.timer;
                pulse.sweep.reset_target(timer);
            }
            0x4002 | 0x4006 => {
                let mut pulse = if address == 0x4002 {
                    self.pulse[0].lock()
                } else {
                    self.pulse[1].lock()
                };
                pulse.timer = (pulse.timer & 0xFF00) | data as u16;
                let timer = pulse.timer;
                pulse.sweep.reset_target(timer);
            }
            0x4003 | 0x4007 => {
                let mut pulse = if address == 0x4003 {
                    self.pulse[0].lock()
                } else {
                    self.pulse[1].lock()
                };
                pulse.length_counter.set_counter(data >> 3);
                pulse.timer = (pulse.timer & 0xFF) | ((data & 0b111) as u16) << 8;
                self.frame_counter = 0;

                pulse.envelope.reload();

                pulse.phase = if address == 0x4003 { 0.0 } else { 0.5 };
                let timer = pulse.timer;
                pulse.sweep.reset_target(timer);
            }
            0x4008 => {
                let mut triangle = self.triangle.lock();
                triangle.length_counter.halt = data & 0x80 > 0;
                triangle.linear_counter.control = data & 0x80 > 0;
                triangle.linear_counter.reload(data & 0x7F);
            }
            0x400A => {
                let mut triangle = self.triangle.lock();
                triangle.timer = (triangle.timer & 0xFF00) | data as u16;
                triangle.linear_counter.reload_current();
            }
            0x400B => {
                let mut triangle = self.triangle.lock();
                triangle.timer = (triangle.timer & 0xFF) | ((data & 0b111) as u16) << 8;
                triangle.length_counter.set_counter(data >> 3);
                triangle.linear_counter.reload_current();
            }
            0x400C => {
                let mut noise = self.noise.lock();
                noise.length_counter.halt = data & 0x20 > 0;

                noise.envelope.should_loop = data & 0x20 > 0;
                noise.envelope.constant_volume = data & 0x10 > 0;
                noise.envelope.envelope = data & 0xF;
                noise.envelope.reload();

                noise.phase = 0.0;
            }
            0x400E => {
                self.noise.lock().set_period(data & 0xF);
            }
            0x400F => {
                let mut noise = self.noise.lock();
                noise.length_counter.set_counter(data >> 3);
                noise.envelope.reload();
            }
            0x4015 => {
                self.pulse[0].lock().set_enabled(data & 1 > 0);
                self.pulse[1].lock().set_enabled(data & 2 > 0);
                self.triangle.lock().set_enabled(data & 4 > 0);
                self.noise.lock().set_enabled(data & 8 > 0);
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
