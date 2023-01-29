mod params;

use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::editor::Editor as VstEditor;
use vst::plugin::{Category, HostCallback, Info, Plugin, PluginParameters};

use jimtel::editor::Editor;
use jimtel::params::Params;
use params::LoudnessLimiterParams;

struct LoudnessLimiter {
    sample_rate_hz: f32,

    loudness: jimtel::loudness::Loudness,
    params: Arc<LoudnessLimiterParams>,

    power_envelope: jimtel::envelope::Envelope,
    loudness_power_envelope: jimtel::envelope::Envelope,

    delay_buffer: jimtel::delay_buffer::DelayBuffer,
}

impl Plugin for LoudnessLimiter {
    fn new(_host: HostCallback) -> Self {
        let sample_rate_hz = 48000.0;

        Self {
            sample_rate_hz,

            loudness: jimtel::loudness::Loudness::new(sample_rate_hz, 1, 1),
            params: Arc::new(LoudnessLimiterParams::new()),

            power_envelope: jimtel::envelope::Envelope::new(sample_rate_hz),
            loudness_power_envelope: jimtel::envelope::Envelope::new(sample_rate_hz),

            delay_buffer: jimtel::delay_buffer::DelayBuffer::new(0),
        }
    }

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

        let base_loudness_power = self.params.loudness.get();
        let samples_num_per_loudness_window =
            (self.params.loudness_window.get() / 1000.0 * self.sample_rate_hz) as usize;
        let loudness_attack_ms = self.params.loudness_attack.get();
        let loudness_release_ms = self.params.loudness_release.get();

        let amplitude = self.params.power_from_loudness.get();
        let amplitude_power = amplitude * amplitude;
        let samples_num_per_power_window =
            (self.params.power_window.get() / 1000.0 * self.sample_rate_hz) as usize;
        let power_release_ms = self.params.power_release.get();

        let silence_beyond_power_limit = self.params.silence_beyond_power.get();

        let delay_samples = (self.params.delay.get() / 1000.0 * self.sample_rate_hz) as usize;

        self.loudness.set_samples_num_per_windows(
            samples_num_per_loudness_window,
            samples_num_per_power_window,
        );

        self.power_envelope.set_coefficients(0.0, power_release_ms);
        self.loudness_power_envelope
            .set_coefficients(loudness_attack_ms, loudness_release_ms);

        self.delay_buffer.set_delay(delay_samples);

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            in_left_buffer.get(0),
            in_right_buffer.get(0),
            out_left_buffer.get_mut(0),
            out_right_buffer.get_mut(0),
        ) {
            let (loudness_power, power) = self
                .loudness
                .add_samples(in_left * input_gain, in_right * input_gain);

            let enveloped_power = self.power_envelope.calculate(power);
            let enveloped_loudness_power = self.loudness_power_envelope.calculate(loudness_power);
            let base_power = enveloped_loudness_power * amplitude_power;

            let loudness_coefficient = (base_loudness_power / enveloped_loudness_power).min(1.0);

            let power_limit_coefficient =
                if silence_beyond_power_limit > 0.5 && enveloped_power > base_power {
                    0.0
                } else {
                    (base_power / enveloped_power).min(1.0)
                };

            let gain =
                input_gain * output_gain * (loudness_coefficient * power_limit_coefficient).sqrt();

            let (delayed_in_left, delayed_in_right) = self.delay_buffer.add(*in_left, *in_right);
            *out_left = delayed_in_left * gain;
            *out_right = delayed_in_right * gain;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }

    fn get_editor(&mut self) -> Option<Box<dyn VstEditor>> {
        Some(Box::new(Editor::new(
            "Jimtel Loudness Limiter".to_string(),
            1280.0,
            640.0,
            self.params.clone(),
        )))
    }
}

vst::plugin_main!(LoudnessLimiter);
