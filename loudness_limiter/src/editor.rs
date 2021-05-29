use std::os::raw::c_void;
use std::sync::Arc;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::{CentralPanel, CtxRef, Grid, Style};
use egui_baseview::{EguiWindow, Queue, RenderSettings, Settings};
use epaint::text::{FontDefinitions, FontFamily, TextStyle};
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

struct State {
    params: Arc<LoudnessLimiterParams>,
}

impl State {
    fn new(params: Arc<LoudnessLimiterParams>) -> Self {
        State { params }
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

impl Editor for LoudnessLimiterEditor {
    fn size(&self) -> (i32, i32) {
        (1024, 360)
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
                size: Size::new(1024.0, 360.0),
                scale: WindowScalePolicy::ScaleFactor(1.0),
            },
            render_settings: RenderSettings::default(),
        };

        EguiWindow::open_parented(
            &ParentWindowHandle(parent),
            settings,
            State::new(self.params.clone()),
            |egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut State| {
                let mut fonts = FontDefinitions::default();
                fonts
                    .family_and_size
                    .insert(TextStyle::Body, (FontFamily::Proportional, 28.0));
                fonts
                    .family_and_size
                    .insert(TextStyle::Button, (FontFamily::Proportional, 28.0));
                fonts
                    .family_and_size
                    .insert(TextStyle::Monospace, (FontFamily::Monospace, 28.0));
                egui_ctx.set_fonts(fonts);

                let mut style: Style = (*egui_ctx.style()).clone();
                style.spacing.slider_width = 700.0;
                style.spacing.item_spacing.y = 16.0;
                egui_ctx.set_style(style);
            },
            |egui_ctx: &CtxRef, _queue: &mut Queue, state: &mut State| {
                CentralPanel::default().show(&egui_ctx, |ui| {
                    Grid::new("root grid").show(ui, |ui| {
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
