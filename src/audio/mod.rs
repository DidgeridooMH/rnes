use std::thread::yield_now;

use sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};

mod resampler;
use resampler::Resampler;

const AUDIO_FREQUENCY: usize = 44100;
const TARGET_BUFFER_SIZE: usize = AUDIO_FREQUENCY / 8;
const MIN_BUFFER_SIZE: usize = TARGET_BUFFER_SIZE / 2;

pub struct AudioOutput {
    sound_sampler: AudioStreamOwner,
    resampler: Resampler,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let sdl_context = sdl3::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpec {
            freq: Some(AUDIO_FREQUENCY as i32),
            channels: Some(1),
            format: Some(AudioFormat::f32_sys()),
        };

        let device = audio_subsystem.open_playback_device(&desired_spec).unwrap();
        let sound_sampler = device.open_device_stream(Some(&desired_spec)).unwrap();
        sound_sampler.resume().unwrap();

        Self {
            sound_sampler,
            resampler: Resampler::default(),
        }
    }
}

impl AudioOutput {
    pub fn push_sample(&mut self, sample: f32) {
        self.resampler.push(sample);

        let queued_samples = self.sound_sampler.queued_bytes().unwrap() as usize / size_of::<f32>();
        if self.resampler.len() > MIN_BUFFER_SIZE && queued_samples < TARGET_BUFFER_SIZE {
            self.flush_audio();
        }

        while self.sound_sampler.queued_bytes().unwrap() as usize / size_of::<f32>()
            > TARGET_BUFFER_SIZE / 2
        {
            yield_now();
        }
    }

    fn flush_audio(&mut self) {
        let desired_samples = self.resampler.len() / 41;
        let resampled_audio = self.resampler.resample(desired_samples as usize);
        self.sound_sampler.put_data_f32(&resampled_audio).unwrap();
    }
}
