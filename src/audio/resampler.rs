use std::{collections::VecDeque, f32::consts::PI};

#[derive(Default)]
pub struct Resampler {
    audio_buffer: Vec<f32>,
    delay_buffer: VecDeque<f32>,
}

impl Resampler {
    pub fn push(&mut self, sample: f32) {
        self.audio_buffer.push(sample);
    }

    pub fn len(&self) -> usize {
        self.audio_buffer.len()
    }

    pub fn resample(&mut self, desired_samples: usize) -> Vec<f32> {
        let mut output = Vec::with_capacity(desired_samples);
        let ratio = self.audio_buffer.len() as f32 / desired_samples as f32;

        for i in 0..desired_samples {
            let center = i as f32 * ratio;
            let sample = self.lanczos_resample(center, ratio);
            output.push(sample);
            self.delay_buffer.push_front(sample);

            let other_sample = if self.delay_buffer.len() == 256 {
                self.delay_buffer.pop_back().unwrap_or(0.0)
            } else {
                0.0
            };

            output.push(other_sample);
        }
        self.audio_buffer.clear();

        output
    }

    fn lanczos_resample(&self, position: f32, ratio: f32) -> f32 {
        const FILTER_SIZE: f32 = 3.0;

        let filter_scale = ratio.max(1.0);
        let filter_width = FILTER_SIZE * filter_scale;

        let center_index = position.floor() as i32;
        let mut result = 0.0;
        let mut weight_sum = 0.0;

        let kernel_range = (filter_width.ceil() as i32) + 1;

        for offset in -kernel_range..=kernel_range {
            let sample_index = center_index + offset;

            if sample_index < 0 || sample_index >= self.audio_buffer.len() as i32 {
                continue;
            }

            let x = (position - sample_index as f32) / filter_scale;
            let weight = self.lanczos_kernel(x, filter_width);

            result += self.audio_buffer[sample_index as usize] * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            result / weight_sum
        } else {
            0.0
        }
    }

    fn lanczos_kernel(&self, x: f32, filter_width: f32) -> f32 {
        if x.abs() < 0.0001 {
            return 1.0;
        }

        if x.abs() >= filter_width {
            return 0.0;
        }

        let pi_x = PI * x;
        let sinc_x = pi_x.sin() / pi_x;

        let pi_x_a = pi_x / filter_width;
        let sinc_x_a = pi_x_a.sin() / pi_x_a;

        sinc_x * sinc_x_a
    }
}
