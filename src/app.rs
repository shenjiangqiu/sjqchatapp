use std::collections::VecDeque;

use iced::{
    alignment::Horizontal,
    button,
    container::{self, Style},
    futures::{channel::mpsc::Sender, SinkExt},
    text_input, Alignment, Application, Button, Column, Command, Container, Font, Length, Row,
    Text,
};

use crate::net::{self, Connection};
enum TextMessage {
    SelfSend(String),
    OtherSend(String),
}
pub struct App {
    input_state: text_input::State,
    input_value: String,

    name_state: text_input::State,
    name_value: String,

    bt_state: button::State,
    ex_bt_state: button::State,
    cl_bt_state: button::State,
    show_value: VecDeque<TextMessage>,
    should_exit: bool,
    connect_status: AppStatus,
}
pub enum AppStatus {
    Disconnected,
    Connected(Sender<String>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Input(String),
    User(String),
    Name(String),
    Submit,
    Exit,
    Clear,
    FinishedSend,
    Connected(Sender<String>),
    Disconnected,
}
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};
const INPUT_FONT: Font = Font::External {
    name: "Input",
    bytes: include_bytes!("../fonts/zenhei.ttf"),
};
impl Application for App {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = ();

    fn title(&self) -> String {
        "sjq chat".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Input(value) => {
                self.input_value = value;
                Command::none()
            }
            Message::User(value) => {
                self.show_value
                    .push_back(TextMessage::OtherSend(format!("recv: {}", value)));
                if self.show_value.len() > 10 {
                    self.show_value.pop_front();
                }
                Command::none()
            }
            Message::Submit => match self.connect_status {
                AppStatus::Disconnected => {
                    println!("connecting...");
                    Command::none()
                }
                AppStatus::Connected(ref mut sender) => {
                    let message = format!(
                        "{} says: {}",
                        if self.name_value.is_empty() {
                            "nobody"
                        } else {
                            &self.name_value
                        },
                        if self.input_value.is_empty() {
                            "nothing"
                        } else {
                            &self.input_value
                        }
                    );
                    self.show_value
                        .push_back(TextMessage::SelfSend(message.clone()));
                    if self.show_value.len() > 10 {
                        self.show_value.pop_front();
                    }
                    self.input_value.clear();
                    let mut sender = sender.clone();
                    Command::perform(async move { sender.send(message).await }, |send_result| {
                        match send_result {
                            Ok(_) => Message::FinishedSend,
                            Err(_) => Message::Disconnected,
                        }
                    })
                }
            },
            Message::Exit => {
                self.should_exit = true;
                Command::none()
            }
            Message::Clear => {
                self.show_value.clear();
                Command::none()
            }
            Message::FinishedSend => {
                println!("send finished");
                Command::none()
            }
            Message::Connected(sender) => {
                println!("connected");
                self.connect_status = AppStatus::Connected(sender);
                Command::none()
            }
            Message::Disconnected => {
                println!("disconnected");
                self.connect_status = AppStatus::Disconnected;
                Command::none()
            }
            Message::Name(name) => {
                self.name_value = name;
                Command::none()
            }
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let input = iced::text_input::TextInput::new(
            &mut self.input_state,
            "input here",
            &self.input_value,
            Message::Input,
        )
        .on_submit(Message::Submit)
        .padding(20)
        .width(Length::FillPortion(20))
        .font(INPUT_FONT);
        let name = iced::text_input::TextInput::new(
            &mut self.name_state,
            "your name",
            &self.name_value,
            Message::Name,
        )
        .width(Length::FillPortion(5))
        .padding(20)
        .font(INPUT_FONT);
        let bt_txt = match self.connect_status {
            AppStatus::Connected(_) => "send",
            AppStatus::Disconnected => "cannot send! disconnected",
        };
        let bt = Button::new(&mut self.bt_state, Text::new(bt_txt))
            .on_press(Self::Message::Submit)
            .padding(5);
        let row = Row::new()
            .push(name)
            .push(input)
            .push(bt)
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center);
        let column = Column::new();

        let column = column.push(row).height(Length::Fill).width(Length::Fill);
        let column = if self.show_value.is_empty() {
            column.push(Text::new("no message").font(INPUT_FONT))
        } else {
            self.show_value
                .iter()
                .enumerate()
                .fold(column, |acc, (idx, item)| {
                    let (text, location_right) = match item {
                        TextMessage::SelfSend(text) => (text, false),
                        TextMessage::OtherSend(text) => (text, true),
                    };
                    let container_style: Box<dyn container::StyleSheet> = if location_right {
                        super::style::ContainerRecv.into()
                    } else {
                        super::style::ContainerSend.into()
                    };
                    acc.push(
                        Container::new(
                            Text::new(format!("{} :{}", idx + 1, text)).font(INPUT_FONT),
                        )
                        .padding(10)
                        .width(Length::Fill)
                        .align_x(if location_right {
                            Horizontal::Right
                        } else {
                            Horizontal::Left
                        })
                        .style(container_style),
                    )
                })
        };

        let column = column.push(
            Row::new()
                .push(
                    Button::new(&mut self.ex_bt_state, Text::new("exit the app!"))
                        .on_press(Self::Message::Exit)
                        .padding(5),
                )
                .push(
                    Button::new(&mut self.cl_bt_state, Text::new("clear the history!"))
                        .on_press(Self::Message::Clear)
                        .padding(5),
                )
                .spacing(3)
                .padding(5),
        );
        Container::new(column.padding(30).spacing(2))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                input_state: Default::default(),
                input_value: "".to_string(),
                show_value: vec![].into(),
                bt_state: Default::default(),
                ex_bt_state: Default::default(),
                cl_bt_state: Default::default(),
                should_exit: false,
                connect_status: AppStatus::Disconnected,
                name_state: Default::default(),
                name_value: Default::default(),
            },
            iced::Command::none(),
        )
    }
    fn should_exit(&self) -> bool {
        self.should_exit
    }
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        net::connect().map(|ev| match ev {
            net::Event::Connected(Connection(sender)) => Message::Connected(sender),
            net::Event::Disconnected => Message::Disconnected,
            net::Event::MessageReceived(msg) => Message::User(msg),
        })
    }
}
