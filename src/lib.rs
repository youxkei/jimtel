use lv2::prelude::*;

#[derive(PortCollection)]
struct Ports {
    input1: InputPort<Audio>,
    input2: InputPort<Audio>,
    output1: OutputPort<Audio>,
    output2: OutputPort<Audio>,

    gain: InputPort<Control>,
}

#[uri("https://github.com/youxkei/jimtel")]
struct Amp;

impl Plugin for Amp {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self)
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _sample_count: u32) {
        let coef = if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        for (in_frame, out_frame) in Iterator::zip(ports.input1.iter(), ports.output1.iter_mut()) {
            *out_frame = in_frame * coef;
        }
    }
}

lv2_descriptors!(Amp);
