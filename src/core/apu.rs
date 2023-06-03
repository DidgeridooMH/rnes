use crate::core::Addressable;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

const DUTY_CYCLES: [[i32; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 0],
];

struct Pulse {
    frequency: f32,
    spec_freq: f32,
    phase: f32,
    volume: f32,

    duty: &'static [i32; 8],
    duty_counter: usize,
}

impl AudioCallback for Pulse {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.duty[self.duty_counter] > 0 {
                self.volume
            } else {
                0.0
            };
            self.duty_counter = (self.duty_counter + 1) % 8;
        }
        /*let phase_inc = self.frequency / self.spec_freq;
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + phase_inc) % 1.0;
        }*/
    }
}

pub struct APU {
    pulse: AudioDevice<Pulse>,
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

        let pulse_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| Pulse {
                frequency: 440.0,
                spec_freq: spec.freq as f32,
                phase: 0.0,
                volume: 0.3,
                duty: &DUTY_CYCLES[2],
                duty_counter: 0,
            })
            .unwrap();
        pulse_device.resume();

        Self {
            pulse: pulse_device,
        }
    }
}

impl Addressable for APU {
    fn read_byte(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, _address: u16, _data: u8) {}
}
