use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

#[derive(params::Params)]
struct LoudnessLimiterParams {
    #[param(unit = "dB", min = "-80", max = "80")]
    input_gain: AtomicFloat,

    #[param(unit = "dB", min = "-80", max = "80")]
    output_gain: AtomicFloat,

    #[param(unit = "LKFS", min = "-80", max = "0")]
    limit: AtomicFloat,

    #[param(unit = "dBFS", min = "-80", max = "0")]
    hard_limit: AtomicFloat,

    #[param(unit = "ms", min = "0", max = "5000")]
    attack: AtomicFloat,

    #[param(unit = "ms", min = "0", max = "5000")]
    release: AtomicFloat,
}

struct LoudnessLimiter {
    loudness: jimtel::loudness::Loudness,

    params: Arc<LoudnessLimiterParams>,
}

impl Default for LoudnessLimiter {
    fn default() -> Self {
        let sample_rate_hz = 48000.0;
        let loudness = jimtel::loudness::Loudness::new(sample_rate_hz, 10.0);

        Self {
            loudness,
            params: Arc::new(LoudnessLimiterParams {
                input_gain: AtomicFloat::new(0.0),
                output_gain: AtomicFloat::new(0.0),
                limit: AtomicFloat::new(0.0),
                hard_limit: AtomicFloat::new(0.0),
                attack: AtomicFloat::new(1000.0),
                release: AtomicFloat::new(1000.0),
            }),
        }
    }
}

impl Plugin for LoudnessLimiter {
    fn get_info(&self) -> Info {
        Info {
            name: "Jimtel Loudness Limiter".to_string(),
            unique_id: 2065809688,
            inputs: 2,
            outputs: 2,
            parameters: LoudnessLimiterParams::num_params() as i32,
            category: Category::Mastering,

            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (input_buffer, output_buffer) = buffer.split();
        let (in_left_buffer, in_right_buffer) = input_buffer.split_at(1);
        let (mut out_left_buffer, output_buffer) = output_buffer.split_at_mut(1);
        let (mut out_right_buffer, _output_buffer) = output_buffer.split_at_mut(1);

        let input_gain = self.params.input_gain.get();
        let output_gain = self.params.output_gain.get();
        let limit = self.params.limit.get();
        let hard_limit = self.params.hard_limit.get();
        let attack_ms = self.params.attack.get();
        let release_ms = self.params.release.get();

        self.loudness
            .set_params(limit, hard_limit, attack_ms, release_ms);

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            in_left_buffer.get(0),
            in_right_buffer.get(0),
            out_left_buffer.get_mut(0),
            out_right_buffer.get_mut(0),
        ) {
            let left_sample = in_left * input_gain;
            let right_sample = in_right * input_gain;

            let (left_sample, right_sample) = self.loudness.add_samples(left_sample, right_sample);

            *out_left = left_sample * output_gain;
            *out_right = right_sample * output_gain;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

vst::plugin_main!(LoudnessLimiter);
