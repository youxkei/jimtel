use std::sync::Arc;

use vst::util::AtomicFloat;

use iced_baseview::{
    executor, slider, text_input, Application, Color, Column, Command, Element, Length, Row,
    Slider, Text, TextInput,
};

use crate::params::LoudnessLimiterParams;

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(i32, f32),
    TextChanged(i32, String),
}

impl Message {
    fn slider_changed(index: i32) -> Box<dyn Fn(f32) -> Message> {
        Box::new(move |value| Message::SliderChanged(index, value))
    }

    fn text_changed(index: i32) -> Box<dyn Fn(String) -> Message> {
        Box::new(move |value| Message::TextChanged(index, value))
    }
}

pub struct Flags {
    pub params: Arc<LoudnessLimiterParams>,
}

struct ParamUI {
    params: Arc<LoudnessLimiterParams>,
    index: i32,
    text: Option<String>,

    slider_state: slider::State,
    text_input_state: text_input::State,
}

impl ParamUI {
    fn new(params: Arc<LoudnessLimiterParams>, index: i32) -> Self {
        Self {
            params,
            index,
            text: None,
            slider_state: Default::default(),
            text_input_state: Default::default(),
        }
    }

    fn view(&mut self) -> Row<Message> {
        if let None = self.text {
            self.text = Some(self.params.get_value_text(self.index))
        }

        Row::new()
            .spacing(8)
            .push(
                Text::new(&self.params.get_name(self.index))
                    .size(32)
                    .width(Length::Units(128)),
            )
            .push(
                Slider::new(
                    &mut self.slider_state,
                    self.params.get_range(self.index),
                    self.params.get_value(self.index),
                    Message::slider_changed(self.index),
                )
                .step(0.1)
                .height(32),
            )
            .push(
                TextInput::new(
                    &mut self.text_input_state,
                    "",
                    self.text.as_ref().unwrap(),
                    Message::text_changed(self.index),
                )
                .size(32)
                .width(Length::Units(96)),
            )
            .push(
                Text::new(self.params.get_unit(self.index))
                    .size(32)
                    .width(Length::Units(64)),
            )
    }
}

pub struct LoudnessLimiterUI {
    params: Arc<LoudnessLimiterParams>,

    param_uis: Vec<ParamUI>,
}

impl Application for LoudnessLimiterUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let params = flags.params;

        let param_uis: Vec<ParamUI> = LoudnessLimiterParams::index_range()
            .map(|index| ParamUI::new(params.clone(), index))
            .collect();

        (
            Self {
                params: params.clone(),
                param_uis,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SliderChanged(index, value) => {
                self.params.set_value(index, value);
                self.param_uis[index as usize].text = Some(self.params.get_value_text(index))
            }

            Message::TextChanged(index, value) => match value.parse::<f32>() {
                Ok(parsed_value) if !value.ends_with(".") => {
                    let range = self.params.get_range(index);
                    let value = parsed_value.min(*range.end()).max(*range.start());

                    self.params.set_value(index, value);
                    self.param_uis[index as usize].text = Some(value.to_string());
                }

                _ => {
                    self.param_uis[index as usize].text = Some(value);
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let mut column = Column::new().padding(16).spacing(16);

        for param_ui in self.param_uis.iter_mut() {
            column = column.push(param_ui.view())
        }

        column.into()
    }

    fn background_color(&self) -> Color {
        Color::WHITE
    }
}
