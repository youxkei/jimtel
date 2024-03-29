use vst::util::AtomicFloat;

use params_derive::Params;

#[derive(Params)]
pub struct LoudnessCeilingParams {
    #[param(kind = "dB", min = "-80", max = "80")]
    pub input_gain: AtomicFloat,

    #[param(kind = "dB", min = "-80", max = "80")]
    pub output_gain: AtomicFloat,

    #[param(kind = "LKFS", min = "-80", max = "0")]
    pub limit: AtomicFloat,

    #[param(kind = "dBFS", min = "-80", max = "0")]
    pub hard_limit: AtomicFloat,

    #[param(kind = "ms", min = "0", max = "5000")]
    pub attack: AtomicFloat,

    #[param(kind = "button", min = "0", max = "1")]
    pub reset: AtomicFloat,
}

impl LoudnessCeilingParams {
    pub fn new() -> Self {
        Self {
            input_gain: AtomicFloat::new(0.0),
            output_gain: AtomicFloat::new(0.0),
            limit: AtomicFloat::new(0.0),
            hard_limit: AtomicFloat::new(0.0),
            attack: AtomicFloat::new(1000.0),
            reset: AtomicFloat::new(0.0),
        }
    }
}
