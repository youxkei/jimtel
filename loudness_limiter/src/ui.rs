use std::sync::Arc;

use iced_baseview::{executor, slider, Application, Color, Command, Element, Row, Slider, Text};
use vst::plugin::PluginParameters;

use crate::params::LoudnessLimiterParams;

#[derive(Debug, Clone)]
pub enum Message {
    InputGainChanged(f32),
}

pub struct Flags {
    pub params: Arc<LoudnessLimiterParams>,
}

pub struct LoudnessLimiterUI {
    params: Arc<LoudnessLimiterParams>,

    input_gain_slider_state: slider::State,
}

impl Application for LoudnessLimiterUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                params: flags.params,
                input_gain_slider_state: Default::default(),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputGainChanged(value) => {
                self.params.input_gain.set(10f32.powf(value * 0.05))
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Row::new()
            .padding(20)
            .spacing(8)
            .push(Text::new("Input Gain").size(32))
            .push(Slider::new(
                &mut self.input_gain_slider_state,
                -80.0..=80.0,
                20.0 * self.params.input_gain.get().log10(),
                Message::InputGainChanged,
            ))
            .push(Text::new(format!("{} dB", self.params.get_parameter_text(0))).size(32))
            .into()
    }

    fn background_color(&self) -> Color {
        Color::WHITE
    }
}
