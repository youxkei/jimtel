#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use baseplug::{Plugin, ProcessContext};
use serde::{Deserialize, Serialize};

baseplug::model! {
    #[derive(Serialize, Deserialize)]
    struct LoudnessLimiterModel {
        #[model(min = -80.0, max = 80.0)]
        #[parameter(name = "Input Gain", unit = "Decibels")]
        input_gain: f32,

        #[model(min = -80.0, max = 80.0)]
        #[parameter(name = "Output Gain", unit = "Decibels")]
        output_gain: f32,

        #[model(min = -80.0, max = 0.0)]
        #[parameter(name = "Limit", unit = "Decibels")]
        limit: f32,

        #[model(min = 0.0, max = 5000.0)]
        #[parameter(name = "Attack", unit = "Generic")]
        attack: f32,

        #[model(min = 0.0, max = 5000.0)]
        #[parameter(name = "Release", unit = "Generic")]
        release: f32,
    }
}

impl Default for LoudnessLimiterModel {
    fn default() -> Self {
        Self {
            input_gain: 1.0,
            output_gain: 1.0,
            limit: 1.0,
            attack: 1000.0,
            release: 1000.0,
        }
    }
}

struct LoudnessLimiter {
    loudness: jimtel::loudness::Loudness,
}

impl Plugin for LoudnessLimiter {
    const NAME: &'static str = "Jimtel Loudness Limiter";
    const PRODUCT: &'static str = "Jimtel Loudness Limiter";
    const VENDOR: &'static str = "Hisayuki Mima <youxkei@gmail>";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = LoudnessLimiterModel;

    #[inline]
    fn new(sample_rate_hz: f32, _model: &LoudnessLimiterModel) -> Self {
        Self {
            loudness: jimtel::loudness::Loudness::new(sample_rate_hz, 50.0),
        }
    }

    #[inline]
    fn process(&mut self, model: &LoudnessLimiterModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        self.loudness
            .set_params(model.limit[0], model.attack[0], model.release[0]);

        for i in 0..ctx.nframes {
            let left_sample = input[0][i] * model.input_gain[0];
            let right_sample = input[1][i] * model.input_gain[0];

            let (left_sample, right_sample) = self.loudness.add_samples(left_sample, right_sample);

            output[0][i] = left_sample * model.output_gain[0];
            output[1][i] = right_sample * model.output_gain[0];
        }
    }
}

baseplug::vst2!(LoudnessLimiter, b"2065");
