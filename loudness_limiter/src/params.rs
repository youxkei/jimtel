use vst::util::AtomicFloat;

use params_derive::Params;

#[derive(Params)]
pub struct LoudnessLimiterParams {
    #[param(kind = "dB", min = "-80", max = "80")]
    pub input_gain: AtomicFloat,

    #[param(kind = "dB", min = "-80", max = "80")]
    pub output_gain: AtomicFloat,

    #[param(kind = "LKFS", min = "-80", max = "0")]
    pub loudness: AtomicFloat,

    #[param(kind = "ms", min = "1", max = "1000")]
    pub loudness_window: AtomicFloat,

    #[param(kind = "ms", min = "0", max = "1000")]
    pub loudness_attack: AtomicFloat,

    #[param(kind = "ms", min = "0", max = "1000")]
    pub loudness_release: AtomicFloat,

    #[param(kind = "dB", min = "0", max = "80")]
    pub power_from_loudness: AtomicFloat,

    #[param(kind = "ms", min = "1", max = "32")]
    pub power_window: AtomicFloat,

    #[param(kind = "ms", min = "0", max = "10000")]
    pub power_release: AtomicFloat,

    #[param(kind = "checkbox", min = "0", max = "1")]
    pub silence_beyond_power: AtomicFloat,

    #[param(kind = "ms", min = "0", max = "1000")]
    pub delay: AtomicFloat,
}

impl LoudnessLimiterParams {
    pub fn new() -> Self {
        let default_loudness = 10f32.powf((-23.0 + 0.691) / 10.0); // -23LKFS
        let default_power_from_loudness = 10f32.powf(5.0 / 20.0); // 5dB

        Self {
            input_gain: AtomicFloat::new(1.0),  // 0dB
            output_gain: AtomicFloat::new(1.0), // 0dB
            loudness: AtomicFloat::new(default_loudness),
            loudness_window: AtomicFloat::new(1000.0),
            loudness_attack: AtomicFloat::new(50.0),
            loudness_release: AtomicFloat::new(0.0),
            power_from_loudness: AtomicFloat::new(default_power_from_loudness),
            power_window: AtomicFloat::new(6.0),
            power_release: AtomicFloat::new(10000.0),
            silence_beyond_power: AtomicFloat::new(0.0),
            delay: AtomicFloat::new(0.0),
        }
    }
}
