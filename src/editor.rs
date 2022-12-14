use std::marker::{Send, Sync};
use std::os::raw::c_void;
use std::sync::Arc;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::{CentralPanel, Context, FontFamily, FontId, Grid, TextStyle};
use egui_baseview::{EguiWindow, Queue};
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
        (self.width as i32, self.height as i32)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        if self.opened {
            return false;
        }

        let settings = WindowOpenOptions {
            title: self.title.clone(),
            size: Size::new(self.width, self.height),
            scale: WindowScalePolicy::ScaleFactor(1.0),
            gl_config: Some(Default::default()),
        };

        EguiWindow::open_parented(
            &WindowHandle(parent),
            settings,
            State::new(self.params.clone()),
            |ctx: &Context, _queue: &mut Queue, _state: &mut State<Params>| {
                let mut style = (*ctx.style()).clone();

                style.text_styles = [
                    (TextStyle::Body, FontId::new(28.0, FontFamily::Proportional)),
                    (
                        TextStyle::Button,
                        FontId::new(28.0, FontFamily::Proportional),
                    ),
                    (
                        TextStyle::Monospace,
                        FontId::new(28.0, FontFamily::Monospace),
                    ),
                ]
                .into();

                style.spacing.slider_width = 700.0;
                style.spacing.item_spacing.y = 16.0;

                ctx.set_style(style);
            },
            |ctx: &Context, _queue: &mut Queue, state: &mut State<Params>| {
                CentralPanel::default().show(&ctx, |ui| {
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
                            } else if state.params.is_checkbox(index) {
                                let mut checked = value > 0.5;
                                if ui
                                    .checkbox(&mut checked, state.params.get_name(index))
                                    .changed()
                                {
                                    if checked {
                                        state.params.set_value(index, 1.0);
                                    } else {
                                        state.params.set_value(index, 0.0);
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
