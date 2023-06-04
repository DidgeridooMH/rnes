mod pulse;
use pulse::Pulse;
mod envelope;
mod sweep;
use sweep::SweepSetup;

use std::time::{Duration, Instant};

use crate::core::Addressable;
use sdl2::audio::{AudioDevice, AudioSpecDesired};

pub const MASTER_VOLUME: f32 = 0.05;
const FRAME_COUNTER_FREQ: usize = 1789773 / 240;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct APU {
    pulse: [AudioDevice<Pulse>; 2],
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
            freq: Some(44100),
            channels: Some(2),
            samples: Some(64),
        };

        let pulse_device0 = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                Pulse::new(spec.freq as f32, 0.0, 0)
            })
            .unwrap();
        pulse_device0.resume();

        let pulse_device1 = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                Pulse::new(spec.freq as f32, 0.5, 1)
            })
            .unwrap();
        pulse_device1.resume();

        Self {
            pulse: [pulse_device0, pulse_device1],
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
        }

        if self.frame_counter == 1
            || (self.sequencer_mode && self.frame_counter == 4)
            || (!self.sequencer_mode && self.frame_counter == 3)
        {
            self.pulse[0].lock().step_length_counter();
            self.pulse[1].lock().step_length_counter();
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
            if self.pulse[0].lock().length_counter > 0 {
                status |= 1;
            }
            if self.pulse[1].lock().length_counter > 0 {
                status |= 2;
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
                pulse.length_counter_halt = data & 0x20 > 0;
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
                pulse.length_counter = LENGTH_TABLE[(data >> 3) as usize];
                pulse.timer = (pulse.timer & 0xFF) | ((data & 0b111) as u16) << 8;
                self.frame_counter = 0;

                pulse.envelope.reload();

                pulse.phase = if address == 0x4003 { 0.0 } else { 0.5 };
                let timer = pulse.timer;
                pulse.sweep.reset_target(timer);
            }
            0x4015 => {
                self.pulse[0].lock().enabled = data & 1 > 0;
                self.pulse[1].lock().enabled = data & 2 > 0;
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
