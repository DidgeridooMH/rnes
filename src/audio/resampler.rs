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
        for i in 0..desired_samples {
            let source_start = i * self.audio_buffer.len() / desired_samples;
            let source_end = (i + 1) * self.audio_buffer.len() / desired_samples;
            if source_start == source_end {
                output.push(self.audio_buffer[source_start]);
            } else {
                let sample = self.audio_buffer[source_start..source_end]
                    .iter()
                    .sum::<f32>()
                    / (source_end - source_start) as f32;
                output.push(sample);
            }
        }
        self.audio_buffer.clear();

        output
    }
}
