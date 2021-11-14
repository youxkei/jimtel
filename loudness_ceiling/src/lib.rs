mod params;

use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::editor::Editor as VstEditor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use jimtel::editor::Editor;
use jimtel::params::Params;
use params::LoudnessCeilingParams;

struct LoudnessCeiling {
    loudness: jimtel::loudness::Loudness,
    params: Arc<LoudnessCeilingParams>,

    envelope: jimtel::envelope::Envelope,
    max_loundess: f32,
    coefficient: f32,
    previous_reset: bool,
}

impl Default for LoudnessCeiling {
    fn default() -> Self {
        let sample_rate_hz = 48000.0;
        let samples_num_per_window = (sample_rate_hz * 3.0) as usize;
        let samples_num_per_calculation = (sample_rate_hz * 0.1) as usize;

        Self {
            loudness: jimtel::loudness::Loudness::new(
                sample_rate_hz,
                samples_num_per_window,
                samples_num_per_calculation,
            ),
            params: Arc::new(LoudnessCeilingParams::new()),

            envelope: jimtel::envelope::Envelope::new(sample_rate_hz),
            max_loundess: 0.0,
            coefficient: 1.0,
            previous_reset: false,
        }
    }
}

impl Plugin for LoudnessCeiling {
    fn get_info(&self) -> Info {
        Info {
            name: "Jimtel Loudness Ceiling".to_string(),
            unique_id: 291815611,
            inputs: 2,
            outputs: 2,
            parameters: LoudnessCeilingParams::num_params() as i32,
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
        let attack_ms = self.params.attack.get();
        let reset = self.params.reset.get() < 0.5;

        self.envelope.set_coefficients(attack_ms, 0.0);

        if reset != self.previous_reset {
            self.max_loundess = limit;
            self.previous_reset = reset;
        }

        self.coefficient = limit / self.max_loundess;

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            in_left_buffer.get(0),
            in_right_buffer.get(0),
            out_left_buffer.get_mut(0),
            out_right_buffer.get_mut(0),
        ) {
            let loudness = self
                .loudness
                .add_samples(in_left * input_gain, in_right * input_gain);
            let loudness = self.envelope.calculate(loudness);

            if loudness > self.max_loundess {
                self.max_loundess = loudness;

                if loudness > limit {
                    self.coefficient = limit / loudness;
                } else {
                    self.coefficient = 1.0;
                }
            }

            let gain = input_gain * output_gain * self.coefficient;

            *out_left = (in_left * gain).min(hard_limit).max(-hard_limit);
            *out_right = (in_right * gain).min(hard_limit).max(-hard_limit);
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }

    fn get_editor(&mut self) -> Option<Box<dyn VstEditor>> {
        Some(Box::new(Editor::new(
            "Jimtel Loudness Ceiling".to_string(),
            1024.0,
            360.0,
            self.params.clone(),
        )))
    }
}

vst::plugin_main!(LoudnessCeiling);
