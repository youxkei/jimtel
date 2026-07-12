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

    // min/max on a meter are its plot's display range (Y axis), not a slider range.
    // The pre/post-gain pair shares a group so the editor shows one selectable line.
    #[param(kind = "LKFS", min = "-60", max = "12", meter, group = "input_loudness")]
    pub input_loudness_pre_gain: AtomicFloat,

    #[param(kind = "LKFS", min = "-60", max = "12", meter, group = "input_loudness")]
    pub input_loudness_post_gain: AtomicFloat,

    #[param(kind = "LKFS", min = "-60", max = "12", meter, group = "output_loudness")]
    pub output_loudness_pre_gain: AtomicFloat,

    #[param(kind = "LKFS", min = "-60", max = "12", meter, group = "output_loudness")]
    pub output_loudness_post_gain: AtomicFloat,

    #[param(kind = "dB", min = "-60", max = "1", meter)]
    pub gain_reduction: AtomicFloat,
}

impl LoudnessLimiterParams {
    pub fn new() -> Self {
        // The dev build starts with a hotter input and a lower loudness target.
        let (default_input_gain, default_loudness) = if cfg!(feature = "dev") {
            (
                10f32.powf(20.0 / 20.0),           // 20dB
                10f32.powf((-28.0 + 0.691) / 10.0), // -28LKFS
            )
        } else {
            (
                1.0,                                // 0dB
                10f32.powf((-23.0 + 0.691) / 10.0), // -23LKFS
            )
        };
        let default_power_from_loudness = 10f32.powf(5.0 / 20.0); // 5dB

        Self {
            input_gain: AtomicFloat::new(default_input_gain),
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

            input_loudness_pre_gain: AtomicFloat::new(f32::EPSILON),
            input_loudness_post_gain: AtomicFloat::new(f32::EPSILON),
            output_loudness_pre_gain: AtomicFloat::new(f32::EPSILON),
            output_loudness_post_gain: AtomicFloat::new(f32::EPSILON),
            gain_reduction: AtomicFloat::new(1.0), // 0dB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoudnessLimiterParams;
    use jimtel::params::Params;
    use vst::plugin::PluginParameters;

    #[test]
    fn meters_are_excluded_from_the_vst_parameter_set() {
        assert_eq!(LoudnessLimiterParams::num_params(), 11);
        assert_eq!(LoudnessLimiterParams::index_range(), 0..11);
        assert_eq!(LoudnessLimiterParams::num_meters(), 5);
        assert_eq!(LoudnessLimiterParams::meter_index_range(), 0..5);

        // Bank data must serialize the 11 parameters only, never the meters.
        let params = LoudnessLimiterParams::new();
        let bank: Vec<f32> = rmp_serde::from_read_ref(&params.get_bank_data()).unwrap();
        assert_eq!(bank.len(), 11);
    }

    #[test]
    fn lkfs_meter_reads_back_in_lkfs() {
        let params = LoudnessLimiterParams::new();

        // Store the mean power for -23 LKFS; the meter must report -23 LKFS.
        let power = 10f32.powf((-23.0 + 0.691) / 10.0);
        params.input_loudness_post_gain.set(power);

        let index = 1; // input_loudness_post_gain (second meter)
        assert_eq!(params.get_meter_name(index), "input_loudness_post_gain");
        assert_eq!(params.get_meter_unit(index), "LKFS");
        assert!((params.get_meter_value(index) - (-23.0)).abs() < 1e-3);
        assert_eq!(params.get_meter_value_text(index), "-23");
    }

    #[test]
    fn gain_reduction_meter_reads_back_in_db() {
        let params = LoudnessLimiterParams::new();

        // A 0.5 amplitude coefficient is a 6.02 dB reduction.
        params.gain_reduction.set(0.5);

        let index = 4; // gain_reduction (fifth meter)
        assert_eq!(params.get_meter_name(index), "gain_reduction");
        assert_eq!(params.get_meter_unit(index), "dB");
        assert!((params.get_meter_value(index) - (-6.0206)).abs() < 1e-3);
    }
}
