mod loudness;

/*
use lv2::prelude::*;

const NO_SOUND_THRESHOLD: f32 = 0.0001;
const NO_SOUND_DURATION_SAMPLES: u64 = 10;

#[derive(PortCollection)]
struct Ports {
    input_left: InputPort<Audio>,
    input_right: InputPort<Audio>,
    output_left: OutputPort<Audio>,
    output_right: OutputPort<Audio>,

    input_gain: InputPort<Control>,
    output_gain: InputPort<Control>,
    limit: InputPort<Control>,
    release: InputPort<Control>,
    use_loudness: InputPort<Control>,
    infinite_sustain: InputPort<Control>,
}

#[uri("https://github.com/youxkei/jimtel")]
struct Jimtel {
    sample_rate_hz: f32,
    current_peak: f32,
    current_coefficienet: f32,
    infinite_sustain: bool,
    no_sound_count: u64,

    loudness: loudness::Loudness,
}

impl Plugin for Jimtel {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        let sample_rate_hz = plugin_info.sample_rate() as f32;

        Some(Self {
            sample_rate_hz: sample_rate_hz,
            current_peak: 0.0,
            current_coefficienet: 0.0,
            infinite_sustain: false,
            no_sound_count: 0,

            loudness: loudness::Loudness::new(sample_rate_hz as f32, 0.4, 0.01),
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _sample_count: u32) {
        let input_gain_db = *ports.input_gain;
        let output_gain_db = *ports.output_gain;
        let limit_dbfs = *ports.limit;
        let release_ms = *ports.release;

        let input_gain = 10.0_f32.powf(input_gain_db * 0.05);
        let output_gain = 10.0_f32.powf(output_gain_db * 0.05);
        let total_gain = input_gain * output_gain;
        let limit = 10.0_f32.powf(limit_dbfs * 0.05);
        let release_speed = limit.powf(1000.0 / (self.sample_rate_hz * release_ms));
        let use_loudness = *ports.use_loudness;
        let infinite_sustain = *ports.infinite_sustain > 0.0;

        self.current_peak = self.current_peak.max(limit);
        self.current_coefficienet = limit / self.current_peak;

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            ports.input_left.iter(),
            ports.input_right.iter(),
            ports.output_left.iter_mut(),
            ports.output_right.iter_mut(),
        ) {
            if !self.infinite_sustain && infinite_sustain {
                self.current_peak = limit;
                self.current_coefficienet = 1.0;

                self.loudness.reset()
            }

            self.infinite_sustain = infinite_sustain;

            let sample_abs = in_left.abs().max(in_right.abs()) * input_gain;

            if sample_abs < NO_SOUND_THRESHOLD {
                if self.no_sound_count >= NO_SOUND_DURATION_SAMPLES
                    && (!infinite_sustain || self.current_peak == limit)
                {
                    self.current_peak = limit;
                    self.current_coefficienet = 1.0;

                    self.loudness.reset();
                } else {
                    self.no_sound_count += 1;
                }
            } else {
                self.no_sound_count = 0;
            }

            self.loudness
                .add_samples(*in_left * input_gain, *in_right * input_gain);

            let (peak, no_sound) = match self.loudness.loudness() {
                Some(loudness) => (
                    loudness * use_loudness + sample_abs * (1.0 - use_loudness),
                    false,
                ),

                None => {
                    if infinite_sustain {
                        (0.0, true)
                    } else {
                        (sample_abs, false)
                    }
                }
            };

            if peak > self.current_peak {
                self.current_peak = peak;
                self.current_coefficienet = limit / self.current_peak;
            } else if !self.infinite_sustain
                && peak < self.current_peak
                && self.current_peak > limit
            {
                self.current_peak *= release_speed;

                let max = peak.max(limit);

                if self.current_peak < max {
                    self.current_peak = max
                }

                self.current_coefficienet = limit / self.current_peak;
            }

            let coefficient = total_gain * self.current_coefficienet;

            if no_sound {
                *out_left = 0.0;
                *out_right = 0.0;
            } else {
                *out_left = in_left * coefficient;
                *out_right = in_right * coefficient;
            }
        }
    }
}

lv2_descriptors!(Jimtel);
*/
