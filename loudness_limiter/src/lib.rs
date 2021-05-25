mod editor;
mod params;

use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use editor::LoudnessLimiterEditor;
use params::LoudnessLimiterParams;

struct LoudnessLimiter {
    loudness: jimtel::loudness::Loudness,
    params: Arc<LoudnessLimiterParams>,
}

impl Default for LoudnessLimiter {
    fn default() -> Self {
        let sample_rate_hz = 48000.0;

        Self {
            loudness: jimtel::loudness::Loudness::new(sample_rate_hz, 1),
            params: Arc::new(LoudnessLimiterParams::new()),
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
            preset_chunks: true,

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
        let release_ms = self.params.release.get();

        self.loudness.set_params(limit, hard_limit, 0.0, release_ms);

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
        self.params.clone()
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        Some(Box::new(LoudnessLimiterEditor::new(self.params.clone())))
    }
}

vst::plugin_main!(LoudnessLimiter);
