use std::collections::HashMap;
use std::marker::{Send, Sync};
use std::os::raw::c_void;
use std::sync::Arc;
use std::time::Instant;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::plot::{Line, Plot, PlotPoints};
use egui::{CentralPanel, Color32, Context, FontFamily, FontId, Grid, ScrollArea, TextStyle};
use egui_baseview::{EguiWindow, Queue};
use vst::editor::Editor as VstEditor;

use crate::params::Params as VstParams;
use crate::window_handle::WindowHandle;

// Meter history is sampled on a fixed time grid (independent of frame rate) and
// kept for the whole session so the user can scroll/zoom back to UI launch.
const METER_SAMPLE_INTERVAL_SECS: f64 = 1.0 / 30.0; // ~30 Hz
const METER_MAX_SAMPLES: usize = 600_000; // safety cap (~5.5 h @ 30 Hz)
const METER_MAX_PLOT_POINTS: usize = 2000; // points actually drawn per line
const METER_FOLLOW_WINDOW_SECS: f64 = 30.0; // width of the scrolling "follow" window

// Distinct colors for the lines within a single plot.
const METER_COLORS: [Color32; 5] = [
    Color32::from_rgb(0x4f, 0xc3, 0xf7), // light blue
    Color32::from_rgb(0x42, 0x6c, 0xff), // blue
    Color32::from_rgb(0xff, 0xb3, 0x4f), // orange
    Color32::from_rgb(0x66, 0xbb, 0x6a), // green
    Color32::from_rgb(0xef, 0x53, 0x50), // red
];

struct State<Params> {
    params: Arc<Params>,

    // Per-meter history of [seconds since launch, value], oldest first.
    meter_history: Vec<Vec<[f64; 2]>>,
    start: Instant,
    last_sample: Option<Instant>,

    // Horizontal scroll offset of the timeline scrollbar, in points. Kept across
    // frames because the scrollbar is drawn below the plots but its offset selects
    // the time window the plots above show.
    timeline_offset: f32,

    // For each meter group, the meter index currently selected for display.
    meter_selection: HashMap<String, i32>,
}

impl<Params: VstParams> State<Params> {
    fn new(params: Arc<Params>) -> Self {
        let meter_history = (0..Params::num_meters()).map(|_| Vec::new()).collect();

        // Default each group to its last member (post-gain, declared after pre).
        let mut meter_selection = HashMap::new();
        for index in Params::meter_index_range() {
            let group = params.get_meter_group(index);
            if !group.is_empty() {
                meter_selection.insert(group, index);
            }
        }

        State {
            params,
            meter_history,
            start: Instant::now(),
            last_sample: None,
            timeline_offset: 0.0,
            meter_selection,
        }
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

// Renders the meters as time-series plots, one plot per unit, with a shared
// horizontal scrollbar (at the bottom) acting as a timeline. The x axis reads in
// seconds relative to now: the right edge is 0s (newest) and older data is to the
// left. While the scrollbar handle is at the right edge the view follows the
// newest samples; drag it left to pin the view to an earlier time, all the way
// back to UI launch. The Y axis stays fixed to each meter's display range.
fn render_meters<Params: VstParams>(ui: &mut egui::Ui, state: &mut State<Params>) {
    if Params::num_meters() == 0 {
        return;
    }

    // Group meters by unit, preserving first-seen order.
    let mut groups: Vec<(String, Vec<i32>)> = Vec::new();
    for index in Params::meter_index_range() {
        let unit = state.params.get_meter_unit(index);
        match groups.iter_mut().find(|(existing, _)| *existing == unit) {
            Some((_, indices)) => indices.push(index),
            None => groups.push((unit, vec![index])),
        }
    }

    // Newest timestamp across all meters == "now" (right edge of the timeline).
    let mut t_end = 0.0f64;
    for history in &state.meter_history {
        if let Some(last) = history.last() {
            t_end = t_end.max(last[0]);
        }
    }

    // Map the timeline onto a scrollbar: the visible window is a fixed number of
    // seconds; the content is wide in proportion to the elapsed time.
    let window = METER_FOLLOW_WINDOW_SECS;
    let viewport_w = ui.available_width().max(1.0) as f64;
    let pixels_per_sec = viewport_w / window;
    let content_w = (t_end * pixels_per_sec).max(viewport_w);

    // The plots are drawn first (scrollbar goes at the bottom), so use the offset
    // captured on the previous frame to pick the visible window.
    let offset = state.timeline_offset as f64;
    let t0 = (offset / pixels_per_sec).clamp(0.0, (t_end - window).max(0.0));
    let t1 = t0 + window;

    // Own the params handle so state can be mutated (selection) alongside reads.
    let params = Arc::clone(&state.params);

    ui.separator();

    for (unit, indices) in &groups {
        // Y range is the union of the members' declared display ranges.
        let mut y_min = f32::INFINITY;
        let mut y_max = f32::NEG_INFINITY;
        for &index in indices {
            let range = params.get_meter_range(index);
            y_min = y_min.min(*range.start());
            y_max = y_max.max(*range.end());
        }
        let y_min = y_min as f64;
        let y_max = y_max as f64;

        ui.add_space(8.0);

        // Control / label row above the plot: a selector for each meter group
        // (pick one variant to plot), a plain colored label for ungrouped meters.
        // This doubles as the legend, so the plot's own (overlapping) one is off.
        let mut displayed: Vec<(i32, Color32)> = Vec::new();
        let mut shown_groups: Vec<String> = Vec::new();
        for &index in indices {
            let color = METER_COLORS[displayed.len() % METER_COLORS.len()];
            let mgroup = params.get_meter_group(index);

            if mgroup.is_empty() {
                ui.horizontal(|ui| {
                    ui.colored_label(color, params.get_meter_name(index));
                    ui.label(format!("{} {}", params.get_meter_value_text(index), unit));
                });
                displayed.push((index, color));
            } else if !shown_groups.contains(&mgroup) {
                shown_groups.push(mgroup.clone());

                let variants: Vec<i32> = indices
                    .iter()
                    .cloned()
                    .filter(|&i| params.get_meter_group(i) == mgroup)
                    .collect();
                let mut selected = state
                    .meter_selection
                    .get(&mgroup)
                    .copied()
                    .unwrap_or(variants[0]);

                ui.horizontal(|ui| {
                    ui.colored_label(color, format!("{}:", mgroup));
                    for &v in &variants {
                        let full = params.get_meter_name(v);
                        let label = full
                            .strip_prefix(&format!("{}_", mgroup))
                            .unwrap_or(&full)
                            .to_string();
                        ui.selectable_value(&mut selected, v, label);
                    }
                    ui.label(format!("{} {}", params.get_meter_value_text(selected), unit));
                });

                state.meter_selection.insert(mgroup, selected);
                displayed.push((selected, color));
            }
        }

        // Size the plot to the number of lines actually shown, not every meter
        // in the unit group (only one variant per group is displayed).
        let height = 120.0 + 24.0 * displayed.len() as f32;

        Plot::new(format!("meter plot {}", unit))
            .height(height)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .set_margin_fraction(egui::vec2(0.0, 0.05))
            // Only include_y: this keeps min_auto_bounds invalid on x, so egui's
            // auto_bounds stays enabled and it re-fits the x axis to the fed data
            // every frame (the window scrolls). Passing include_x too would make
            // auto_bounds false and freeze the view at the first frame's bounds.
            .include_y(y_min)
            .include_y(y_max)
            // Label the x axis as seconds ago: 0 at the right, negative to the left.
            .x_axis_formatter(move |x, _range| format!("{:.0}s", x - t_end))
            .show(ui, |plot_ui| {
                // Invisible sentinel pinning the visible window to a constant
                // [t0, t1] x-range, so the window width stays constant and the
                // region before any data existed simply shows as blank.
                plot_ui.line(
                    Line::new(vec![[t0, y_min], [t1, y_min]]).color(Color32::TRANSPARENT),
                );

                for &(index, color) in &displayed {
                    let history = &state.meter_history[index as usize];

                    // Draw only the samples inside the visible window, strided down
                    // to a bounded point count.
                    let lo = history.partition_point(|p| p[0] < t0).saturating_sub(1);
                    let hi = history.partition_point(|p| p[0] <= t1).min(history.len());
                    let visible = &history[lo..hi.max(lo)];
                    let stride = (visible.len() / METER_MAX_PLOT_POINTS).max(1);

                    let points: PlotPoints = visible
                        .iter()
                        .step_by(stride)
                        .map(|p| [p[0], p[1].clamp(y_min, y_max)])
                        .collect();

                    plot_ui.line(Line::new(points).color(color).width(1.5));
                }
            });
    }

    // Timeline scrollbar at the bottom. stick_to_right => follow newest until the
    // user drags it back, and re-follow once dragged to the end again.
    ui.add_space(4.0);
    let scroll = ScrollArea::horizontal()
        .stick_to_right(true)
        .id_source("meter timeline")
        .show(ui, |ui| {
            ui.allocate_exact_size(egui::vec2(content_w as f32, 6.0), egui::Sense::hover());
        });
    state.timeline_offset = scroll.state.offset.x;
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
                if Params::num_meters() > 0 {
                    // Meters are driven by the audio thread. request_repaint()
                    // (not request_repaint_after) is required: this egui-baseview
                    // revision only redraws when egui reports repaint_after == 0,
                    // which request_repaint() produces. Without it the plots only
                    // update while the pointer is moving.
                    ctx.request_repaint();

                    // Append to the history on a fixed time grid, regardless of the
                    // (higher, variable) frame rate.
                    let now = Instant::now();
                    let due = match state.last_sample {
                        None => true,
                        Some(last) => {
                            now.duration_since(last).as_secs_f64() >= METER_SAMPLE_INTERVAL_SECS
                        }
                    };

                    if due {
                        state.last_sample = Some(now);
                        let seconds = now.duration_since(state.start).as_secs_f64();

                        for index in Params::meter_index_range() {
                            let value = state.params.get_meter_value(index) as f64;
                            let history = &mut state.meter_history[index as usize];

                            history.push([seconds, value]);
                            if history.len() > METER_MAX_SAMPLES {
                                history.drain(0..METER_MAX_SAMPLES / 5);
                            }
                        }
                    }
                }

                CentralPanel::default().show(&ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
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

                        render_meters(ui, state);
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
