use std::marker::{Send, Sync};
use std::os::raw::c_void;
use std::sync::Arc;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::{CentralPanel, CtxRef, Grid, Style};
use egui_baseview::{EguiWindow, Queue, RenderSettings, Settings};
use epaint::text::{FontDefinitions, FontFamily, TextStyle};
use vst::editor::Editor as VstEditor;

use crate::params::Params as VstParams;
use crate::window_handle::WindowHandle;

struct State<Params> {
    params: Arc<Params>,
}

impl<Params> State<Params> {
    fn new(params: Arc<Params>) -> Self {
        State { params }
    }
}

pub struct Editor<Params> {
    title: String,
    width: f64,
    height: f64,

    opened: bool,
    params: Arc<Params>,
}

impl<Params> Editor<Params> {
    pub fn new(title: String, width: f64, height: f64, params: Arc<Params>) -> Self {
        Self {
            title,
            width,
            height,
            params,
            opened: false,
        }
    }
}

impl<Params: 'static + VstParams + Send + Sync> VstEditor for Editor<Params> {
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
                title: self.title.clone(),
                size: Size::new(self.width, self.height),
                scale: WindowScalePolicy::ScaleFactor(1.0),
            },
            render_settings: RenderSettings::default(),
        };

        EguiWindow::open_parented(
            &WindowHandle(parent),
            settings,
            State::new(self.params.clone()),
            |egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut State<Params>| {
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
            |egui_ctx: &CtxRef, _queue: &mut Queue, state: &mut State<Params>| {
                CentralPanel::default().show(&egui_ctx, |ui| {
                    Grid::new("root grid").show(ui, |ui| {
                        for index in Params::index_range() {
                            let mut value = state.params.get_value(index);

                            if state.params.is_button(index) {
                                if ui.button(state.params.get_name(index)).clicked() {
                                    if value < 0.5 {
                                        state.params.set_value(index, 1.0)
                                    } else {
                                        state.params.set_value(index, 0.0)
                                    }
                                }
                            } else {
                                ui.label(state.params.get_name(index));

                                if ui
                                    .add(
                                        egui::Slider::new(
                                            &mut value,
                                            state.params.get_range(index),
                                        )
                                        .clamp_to_range(true)
                                        .suffix(state.params.get_unit(index)),
                                    )
                                    .changed()
                                {
                                    state.params.set_value(index, value);
                                }
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
