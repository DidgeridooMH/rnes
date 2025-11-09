#[derive(Default)]
pub struct Resampler {
    audio_buffer: Vec<f32>,
    counter: usize,
}

impl Resampler {
    pub fn push(&mut self, sample: f32) {
        self.audio_buffer.push(sample);
        self.counter += 1;
    }

    pub fn len(&self) -> usize {
        self.audio_buffer.len()
    }

    pub fn resample(&mut self, desired_samples: usize) -> Vec<f32> {
        let mut output = Vec::with_capacity(desired_samples);
        let ratio = self.audio_buffer.len() as f32 / desired_samples as f32;

        for i in 0..desired_samples {
            let center = i as f32 * ratio;
            let sample = self.linear_interpolate(center);
            output.push(sample);
        }
        self.audio_buffer.clear();

        output
    }

    fn linear_interpolate(&self, position: f32) -> f32 {
        let index = position.floor() as usize;

        if index >= self.audio_buffer.len() - 1 {
            return *self.audio_buffer.last().unwrap_or(&0.0);
        }

        self.audio_buffer[index] * (1.0 - position.fract())
            + self.audio_buffer[index + 1] * position.fract()
    }
}
