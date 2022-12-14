pub struct Envelope {
    value: f32,

    sample_rate_hz: f32,
    attack_coefficient: f32,
    release_coefficient: f32,
}

impl Envelope {
    pub fn new(sample_rate_hz: f32) -> Self {
        Envelope {
            value: 0.0,

            sample_rate_hz,
            attack_coefficient: 1.0,
            release_coefficient: 1.0,
        }
    }

    pub fn calculate(&mut self, value: f32) -> f32 {
        if value > self.value {
            self.value = (self.value * self.attack_coefficient).min(value);
        } else if value < self.value {
            self.value = (self.value * self.release_coefficient).max(value);
        }

        self.value = self.value.max(f32::EPSILON); // should be greater than 0

        self.value
    }

    pub fn set_coefficients(&mut self, attack_ms: f32, release_ms: f32) {
        // (+80 / attack_samples) dB
        self.attack_coefficient =
            10f32.powf(80.0 / (attack_ms / 1000.0 * self.sample_rate_hz) / 20.0);

        // (-80 / release_samples) dB
        self.release_coefficient =
            10f32.powf(-80.0 / (release_ms / 1000.0 * self.sample_rate_hz) / 20.0);
    }
}
