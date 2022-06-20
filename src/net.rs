use std::any::TypeId;

use async_tungstenite::{tokio::ConnectStream, tungstenite, WebSocketStream};

use iced::{
    futures::{channel::mpsc, SinkExt, StreamExt}, Subscription,
};
use iced_native::subscription;

pub fn connect() -> Subscription<Event> {
    struct Connect;
    subscription::unfold(
        TypeId::of::<Connect>(),
        State::Disconnected,
        |state| async move {
            match state {
                State::Disconnected => {
                    const ECHO_SERVER: &str = "ws://thesjq.com:3030";
                    println!("try to connect to {}", ECHO_SERVER);
                    match async_tungstenite::tokio::connect_async(ECHO_SERVER).await {
                        Ok((websocket, _)) => {
                            println!("connected!");
                            let (sender, receiver) = mpsc::channel(100);

                            (
                                Some(Event::Connected(Connection(sender))),
                                State::Connected(Box::new(websocket), receiver),
                            )
                        }
                        Err(_) => {
                            println!("connection failed");
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            (Some(Event::Disconnected), State::Disconnected)
                        }
                    }
                }
                State::Connected(mut websocket, mut input) => {
                    let mut fused_websocket = websocket.by_ref().fuse();
                    iced::futures::select! {
                        received = fused_websocket.select_next_some() => {
                            println!("received: {:?}", received);
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                    (
                                        Some(Event::MessageReceived(message)),
                                        State::Connected(websocket, input)
                                    )
                                }
                                Ok(_) => {
                                    (None, State::Connected(websocket, input))
                                }
                                Err(_) => {
                                    (Some(Event::Disconnected), State::Disconnected)
                                }
                            }
                        }

                        message = input.select_next_some() => {
                            println!("message: {:?}", message);
                            let result = websocket.send(tungstenite::Message::Text(message)).await;

                            if result.is_ok() {
                                (None, State::Connected(websocket, input))
                            } else {
                                (Some(Event::Disconnected), State::Disconnected)
                            }
                        }
                    }
                }
            }
        },
    )
}
pub enum State {
    Disconnected,
    Connected(Box<WebSocketStream<ConnectStream>>, mpsc::Receiver<String>),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(String),
}
#[derive(Debug, Clone)]
pub struct Connection(pub mpsc::Sender<String>);
