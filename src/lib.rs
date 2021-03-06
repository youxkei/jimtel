use lv2::prelude::*;

#[derive(PortCollection)]
struct Ports {
    input_left: InputPort<Audio>,
    input_right: InputPort<Audio>,
    output_left: OutputPort<Audio>,
    output_right: OutputPort<Audio>,

    input_gain: InputPort<Control>,
    output_gain: InputPort<Control>,
    limit: InputPort<Control>,
    threshold: InputPort<Control>,
    duration: InputPort<Control>,
}

#[uri("https://github.com/youxkei/jimtel")]
struct Jimtel {
    sample_rate: f64,
    current_peak: f32,
    current_coefficienet: f32,
    samples_under_threshold: i32,
}

impl Plugin for Jimtel {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self {
            sample_rate: plugin_info.sample_rate(),
            current_peak: 0.0,
            current_coefficienet: 0.0,
            samples_under_threshold: 0,
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _sample_count: u32) {
        let input_gain_db = *ports.input_gain;
        let output_gain_db = *ports.output_gain;
        let limit_dbfs = *ports.limit;
        let threshold_dbfs = *ports.threshold;
        let duration_ms = *ports.duration as f64;

        let input_gain = 10.0_f32.powf(input_gain_db * 0.05);
        let output_gain = 10.0_f32.powf(output_gain_db * 0.05);
        let total_gain = input_gain * output_gain;
        let limit = 10.0_f32.powf(limit_dbfs * 0.05);
        let threshold = 10.0_f32.powf(threshold_dbfs * 0.05);

        self.current_peak = self.current_peak.max(limit);
        self.current_coefficienet = limit / self.current_peak;

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            ports.input_left.iter(),
            ports.input_right.iter(),
            ports.output_left.iter_mut(),
            ports.output_right.iter_mut(),
        ) {
            let sample_abs = in_left.abs().max(in_right.abs()) * input_gain;

            if sample_abs < threshold {
                self.samples_under_threshold += 1;

                if self.samples_under_threshold > (self.sample_rate * duration_ms / 1000.0) as i32 {
                    self.current_peak = limit;
                    self.current_coefficienet = 1.0;
                }
            } else {
                self.samples_under_threshold = 0;
            }

            if sample_abs > self.current_peak {
                self.current_peak = sample_abs;
                self.current_coefficienet = limit / self.current_peak;
            }

            let coefficient = total_gain * self.current_coefficienet;

            *out_left = in_left * coefficient;
            *out_right = in_right * coefficient;
        }
    }
}

lv2_descriptors!(Jimtel);
