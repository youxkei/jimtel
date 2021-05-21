mod params;
mod ui;

use std::os::raw::c_void;
use std::sync::Arc;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::IcedWindow;
use raw_window_handle::{unix::XcbHandle, HasRawWindowHandle, RawWindowHandle};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use params::LoudnessLimiterParams;
use ui::{Flags, LoudnessLimiterUI};

struct LoudnessLimiterEditor {
    handle: Option<iced_baseview::WindowHandle<ui::Message>>,
    params: Arc<LoudnessLimiterParams>,
}

struct WindowHandle {
    handle: *mut c_void,
}

unsafe impl HasRawWindowHandle for WindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Xcb(XcbHandle {
            window: self.handle as u32,
            ..XcbHandle::empty()
        })
    }
}

impl Editor for LoudnessLimiterEditor {
    fn size(&self) -> (i32, i32) {
        (1024, 1024)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        let settings = iced_baseview::Settings {
            window: WindowOpenOptions {
                title: "Jimtel Loudness Limiter".to_string(),
                size: Size::new(1024.0, 1024.0),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            flags: Flags {
                params: self.params.clone(),
            },
        };

        let handle = IcedWindow::<LoudnessLimiterUI>::open_parented(
            &WindowHandle { handle: parent },
            settings,
        );

        self.handle = Some(handle);

        true
    }

    fn is_open(&mut self) -> bool {
        self.handle.is_some()
    }

    fn close(&mut self) {
        match self.handle {
            Some(ref mut handle) => handle.close_window().unwrap(),
            None => {}
        };

        self.handle = None;
    }
}

struct LoudnessLimiter {
    loudness: jimtel::loudness::Loudness,

    params: Arc<LoudnessLimiterParams>,
}

impl Default for LoudnessLimiter {
    fn default() -> Self {
        let sample_rate_hz = 48000.0;
        let loudness = jimtel::loudness::Loudness::new(sample_rate_hz, 1);

        Self {
            loudness,
            params: Arc::new(LoudnessLimiterParams::new()),
        }
    }
}

impl Plugin for LoudnessLimiter {
    fn get_info(&self) -> Info {
        Info {
            name: "Jimtel Loudness Limiter".to_string(),
            unique_id: 2065809688,
            inputs: 2,
            outputs: 2,
            parameters: LoudnessLimiterParams::num_params() as i32,
            category: Category::Mastering,

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
        let release_ms = self.params.release.get();

        self.loudness.set_params(limit, hard_limit, 0.0, release_ms);

        for (in_left, in_right, out_left, out_right) in itertools::izip!(
            in_left_buffer.get(0),
            in_right_buffer.get(0),
            out_left_buffer.get_mut(0),
            out_right_buffer.get_mut(0),
        ) {
            let left_sample = in_left * input_gain;
            let right_sample = in_right * input_gain;

            let (left_sample, right_sample) = self.loudness.add_samples(left_sample, right_sample);

            *out_left = left_sample * output_gain;
            *out_right = right_sample * output_gain;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        Some(Box::new(LoudnessLimiterEditor {
            handle: None,
            params: self.params.clone(),
        }))
    }
}

vst::plugin_main!(LoudnessLimiter);
