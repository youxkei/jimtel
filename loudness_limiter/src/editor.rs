use std::os::raw::c_void;
use std::sync::Arc;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::{CentralPanel, CtxRef, Grid, Style};
use egui_baseview::{EguiWindow, Queue, RenderSettings, Settings};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use vst::editor::Editor;

use crate::params::LoudnessLimiterParams;

struct ParentWindowHandle(*mut c_void);

unsafe impl HasRawWindowHandle for ParentWindowHandle {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }

    #[cfg(target_os = "windows")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }

    #[cfg(target_os = "macos")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0,
            ..MacOSHandle::empty()
        })
    }
}

pub struct LoudnessLimiterEditor {
    opened: bool,
    params: Arc<LoudnessLimiterParams>,
}

impl LoudnessLimiterEditor {
    pub fn new(params: Arc<LoudnessLimiterParams>) -> Self {
        Self {
            params,
            opened: false,
        }
    }
}

struct State {
    params: Arc<LoudnessLimiterParams>,
}

impl State {
    fn new(params: Arc<LoudnessLimiterParams>) -> Self {
        State { params }
    }
}

impl Editor for LoudnessLimiterEditor {
    fn size(&self) -> (i32, i32) {
        (1024, 512)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        if self.opened {
            return false;
        }

        let settings = Settings {
            window: WindowOpenOptions {
                title: "Jimtel Loudness Limiter".to_string(),
                size: Size::new(1024.0, 512.0),
                scale: WindowScalePolicy::ScaleFactor(2.0),
            },
            render_settings: RenderSettings::default(),
        };

        EguiWindow::open_parented(
            &ParentWindowHandle(parent),
            settings,
            State::new(self.params.clone()),
            |egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut State| {
                let mut style: Style = (*egui_ctx.style()).clone();
                style.spacing.slider_width *= 3.0;
                egui_ctx.set_style(style);
            },
            |egui_ctx: &CtxRef, _queue: &mut Queue, state: &mut State| {
                CentralPanel::default().show(&egui_ctx, |ui| {
                    Grid::new("root grid").striped(true).show(ui, |ui| {
                        for index in LoudnessLimiterParams::index_range() {
                            let mut value = state.params.get_value(index);

                            ui.label(state.params.get_name(index));

                            if ui
                                .add(
                                    egui::Slider::new(&mut value, state.params.get_range(index))
                                        .clamp_to_range(true)
                                        .suffix(state.params.get_unit(index)),
                                )
                                .changed()
                            {
                                state.params.set_value(index, value);
                            }

                            ui.end_row();
                        }
                    });
                });
            },
        );

        true
    }

    fn is_open(&mut self) -> bool {
        self.opened
    }

    fn close(&mut self) {
        self.opened = false;
    }
}
