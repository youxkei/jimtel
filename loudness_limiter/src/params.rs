use std::sync::Arc;

use vst::util::AtomicFloat;

use params_derive::Params;

#[derive(Params)]
pub struct LoudnessLimiterParams {
    #[param(unit = "dB", min = "-80", max = "80")]
    pub input_gain: Arc<AtomicFloat>,

    #[param(unit = "dB", min = "-80", max = "80")]
    pub output_gain: Arc<AtomicFloat>,

    #[param(unit = "LKFS", min = "-80", max = "0")]
    pub limit: Arc<AtomicFloat>,

    #[param(unit = "dBFS", min = "-80", max = "0")]
    pub hard_limit: Arc<AtomicFloat>,

    #[param(unit = "ms", min = "0", max = "5000")]
    pub release: Arc<AtomicFloat>,
}

impl LoudnessLimiterParams {
    pub fn new() -> Self {
        Self {
            input_gain: Arc::new(AtomicFloat::new(0.0)),
            output_gain: Arc::new(AtomicFloat::new(0.0)),
            limit: Arc::new(AtomicFloat::new(0.0)),
            hard_limit: Arc::new(AtomicFloat::new(0.0)),
            release: Arc::new(AtomicFloat::new(1000.0)),
        }
    }
}
