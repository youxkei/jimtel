use std::f32;

pub struct Loudness {
    samples_num_per_window: usize,
    samples_num_per_calculation: usize,

    left_prefilter: Prefilter,
    right_prefilter: Prefilter,

    sample_buffer: Vec<f32>,
    current_sample: usize,
    count: usize,

    loudness: f32,
}

impl Loudness {
    pub fn new(
        sample_rate_hz: f32,
        samples_num_per_window: usize,
        samples_num_per_calculation: usize,
    ) -> Loudness {
        Loudness {
            samples_num_per_window,
            samples_num_per_calculation,

            left_prefilter: Prefilter::new(sample_rate_hz),
            right_prefilter: Prefilter::new(sample_rate_hz),

            sample_buffer: vec![0.0; samples_num_per_window * 2],
            current_sample: 0,
            count: 0,

            loudness: 0.0,
        }
    }

    pub fn add_samples(&mut self, left_sample: f32, right_sample: f32) -> f32 {
        let left_sample = self.left_prefilter.apply(left_sample);
        let right_sample = self.right_prefilter.apply(right_sample);

        self.sample_buffer[self.current_sample] =
            left_sample * left_sample + right_sample * right_sample;

        self.current_sample += 1;
        if self.current_sample >= self.samples_num_per_window {
            self.current_sample = 0;
        }

        self.count += 1;
        if self.count >= self.samples_num_per_calculation {
            self.count = 0;

            let mut sum = 0.0;
            let mut residue = 0.0;

            for index in self.current_sample..(self.current_sample + self.samples_num_per_window) {
                let index = if index >= self.samples_num_per_window {
                    index - self.samples_num_per_window
                } else {
                    index
                };

                let power = self.sample_buffer[index];

                let tmp = sum + (residue + power);
                residue = (residue + power) - (tmp - sum);
                sum = tmp;
            }

            self.loudness = 0.9235 * (sum / self.samples_num_per_window as f32).sqrt();
        }

        self.loudness
    }
}

struct Prefilter {
    first: Filter,
    second: Filter,
}

impl Prefilter {
    fn new(sample_rate_hz: f32) -> Prefilter {
        Prefilter {
            first: Filter::high_shelf(sample_rate_hz),
            second: Filter::high_pass(sample_rate_hz),
        }
    }

    #[inline(always)]
    fn apply(&mut self, sample: f32) -> f32 {
        self.second.apply(self.first.apply(sample))
    }
}

// struct Filter taken from https://github.com/ruuda/bs1770/blob/db97c508fa68fef3caec649f3ee756a810f2266f/src/lib.rs
struct Filter {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,

    // The past two input and output samples.
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Filter {
    /// Stage 1 of th BS.1770-4 pre-filter.
    fn high_shelf(sample_rate_hz: f32) -> Filter {
        // Coefficients taken from https://github.com/csteinmetz1/pyloudnorm/blob/6baa64d59b7794bc812e124438692e7fd2e65c0c/pyloudnorm/meter.py#L135-L136.
        let gain_db = 3.99984385397;
        let q = 0.7071752369554193;
        let center_hz = 1681.9744509555319;

        // Formula taken from https://github.com/csteinmetz1/pyloudnorm/blob/6baa64d59b7794bc812e124438692e7fd2e65c0c/pyloudnorm/iirfilter.py#L134-L143.
        let k = (f32::consts::PI * center_hz / sample_rate_hz).tan();
        let vh = 10.0_f32.powf(gain_db / 20.0);
        let vb = vh.powf(0.499666774155);
        let a0 = 1.0 + k / q + k * k;
        Filter {
            b0: (vh + vb * k / q + k * k) / a0,
            b1: 2.0 * (k * k - vh) / a0,
            b2: (vh - vb * k / q + k * k) / a0,
            a1: 2.0 * (k * k - 1.0) / a0,
            a2: (1.0 - k / q + k * k) / a0,

            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Stage 2 of th BS.1770-4 pre-filter.
    fn high_pass(sample_rate_hz: f32) -> Filter {
        // Coefficients taken from https://github.com/csteinmetz1/pyloudnorm/blob/6baa64d59b7794bc812e124438692e7fd2e65c0c/pyloudnorm/meter.py#L135-L136.
        let q = 0.5003270373253953;
        let center_hz = 38.13547087613982;

        // Formula taken from https://github.com/csteinmetz1/pyloudnorm/blob/6baa64d59b7794bc812e124438692e7fd2e65c0c/pyloudnorm/iirfilter.py#L145-L151
        let k = (f32::consts::PI * center_hz / sample_rate_hz).tan();
        Filter {
            a1: 2.0 * (k * k - 1.0) / (1.0 + k / q + k * k),
            a2: (1.0 - k / q + k * k) / (1.0 + k / q + k * k),
            b0: 1.0,
            b1: -2.0,
            b2: 1.0,

            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Feed the next input sample, get the next output sample.
    #[inline(always)]
    pub fn apply(&mut self, x0: f32) -> f32 {
        let y0 = 0.0 + self.b0 * x0 + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;

        y0
    }
}
