use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin};

#[derive(Default)]
struct LimiterPlugin;

impl Plugin for LimiterPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Jimtel Limiter".to_string(),
            unique_id: 2065809688,
            inputs: 2,
            outputs: 2,
            category: Category::Mastering,

            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (input_buffer, output_buffer) = buffer.split();
        let (in_left_buffer, in_right_buffer) = input_buffer.split_at(1);
        let (mut out_left_buffer, mut out_right_buffer) = output_buffer.split_at_mut(1);

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            in_left_buffer.get(0),
            in_right_buffer.get(0),
            out_left_buffer.get_mut(0),
            out_right_buffer.get_mut(0),
        ) {
            *out_left = *in_left;
            *out_right = *in_right;
        }
    }
}

vst::plugin_main!(LimiterPlugin);
