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
            self.value += (value - self.value) * self.attack_coefficient;
        } else {
            self.value += (value - self.value) * self.release_coefficient;
        }

        self.value
    }

    pub fn set_coefficients(&mut self, attack_ms: f32, release_ms: f32) {
        self.attack_coefficient = (4000.0 / (attack_ms * self.sample_rate_hz)).min(1.0);
        self.release_coefficient = (4000.0 / (release_ms * self.sample_rate_hz)).min(1.0);
    }
}
