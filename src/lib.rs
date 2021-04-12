use lv2::prelude::*;

mod loudness;

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
    infinite_sustain: InputPort<Control>,
}

#[uri("https://github.com/youxkei/jimtel")]
struct Jimtel {
    sample_rate_hz: f32,
    current_peak: f32,
    current_coefficienet: f32,
    infinite_sustain: bool,

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
        let infinite_sustain = *ports.infinite_sustain > 0.0;
        let release_speed = (1.0 - limit) / (self.sample_rate_hz * release_ms / 1000.0) / 10.0;

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

            let loudness = match self
                .loudness
                .add_samples(*in_left * input_gain, *in_right * input_gain)
            {
                Some(loudness) => loudness,

                None => in_left.abs().max(in_right.abs()) * input_gain,
            };

            if loudness > self.current_peak {
                self.current_peak = loudness;
                self.current_coefficienet = limit / self.current_peak;
            } else if self.current_peak > limit {
                self.current_peak -= release_speed;

                if self.current_peak < limit {
                    self.current_peak = limit
                }

                self.current_coefficienet = limit / self.current_peak;
            }

            let coefficient = total_gain * self.current_coefficienet;

            *out_left = in_left * coefficient;
            *out_right = in_right * coefficient;
        }
    }
}

lv2_descriptors!(Jimtel);
