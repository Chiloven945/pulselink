//! Internal WebSocket transport abstraction.

use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};

use crate::error::WsError;
use crate::options::SocketOptions;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A message written to the WebSocket.
#[derive(Clone, Debug)]
pub(crate) enum Outbound {
    Text(String),
    Pong(Vec<u8>),
    Close { code: Option<u16>, reason: String },
}

/// A message read from the WebSocket.
#[derive(Clone, Debug)]
pub(crate) enum Inbound {
    Text(String),
    Ping(Vec<u8>),
    Pong,
    Close { code: Option<u16>, reason: String },
}

#[async_trait]
pub(crate) trait WsSink: Send {
    async fn send(&mut self, message: Outbound) -> Result<(), WsError>;
}

#[async_trait]
pub(crate) trait WsSource: Send {
    async fn recv(&mut self) -> Option<Result<Inbound, WsError>>;
}

pub(crate) trait WsTransport: Send + Sized + 'static {
    type Sink: WsSink + 'static;
    type Source: WsSource + 'static;

    async fn connect(options: &SocketOptions) -> Result<Self, WsError>;
    fn split(self) -> (Self::Sink, Self::Source);
}

pub(crate) struct TungsteniteTransport {
    stream: WsStream,
}

impl WsTransport for TungsteniteTransport {
    type Sink = SplitSink<WsStream, Message>;
    type Source = SplitStream<WsStream>;

    async fn connect(options: &SocketOptions) -> Result<Self, WsError> {
        if !(options.url.starts_with("ws://") || options.url.starts_with("wss://")) {
            return Err(WsError::InvalidParams(
                "URL must use ws:// or wss://".to_owned(),
            ));
        }
        if options.max_message_size == 0 {
            return Err(WsError::InvalidParams(
                "max_message_size must be positive".to_owned(),
            ));
        }

        let mut config = WebSocketConfig::default();
        config.max_message_size = Some(options.max_message_size);
        config.max_frame_size = Some(options.max_message_size);
        let (stream, _) = timeout(
            options.connect_timeout,
            connect_async_with_config(&options.url, Some(config), false),
        )
        .await
        .map_err(|_| WsError::Transport("websocket connect timed out".to_owned()))?
        .map_err(|error| WsError::Transport(error.to_string()))?;

        Ok(Self { stream })
    }

    fn split(self) -> (Self::Sink, Self::Source) {
        self.stream.split()
    }
}

#[async_trait]
impl WsSink for SplitSink<WsStream, Message> {
    async fn send(&mut self, message: Outbound) -> Result<(), WsError> {
        let message = match message {
            Outbound::Text(text) => Message::Text(text.into()),
            Outbound::Pong(payload) => Message::Pong(payload.into()),
            Outbound::Close { code, reason } => {
                let frame = code.map(|code| CloseFrame {
                    code: CloseCode::from(code),
                    reason: reason.into(),
                });
                Message::Close(frame)
            }
        };
        SinkExt::send(self, message)
            .await
            .map_err(|error| WsError::Transport(error.to_string()))
    }
}

#[async_trait]
impl WsSource for SplitStream<WsStream> {
    async fn recv(&mut self) -> Option<Result<Inbound, WsError>> {
        match self.next().await? {
            Ok(Message::Text(text)) => Some(Ok(Inbound::Text(text.to_string()))),
            Ok(Message::Binary(bytes)) => match String::from_utf8(bytes.to_vec()) {
                Ok(text) => Some(Ok(Inbound::Text(text))),
                Err(error) => Some(Err(WsError::Transport(format!(
                    "binary message is not UTF-8: {error}"
                )))),
            },
            Ok(Message::Ping(payload)) => Some(Ok(Inbound::Ping(payload.to_vec()))),
            Ok(Message::Pong(_)) => Some(Ok(Inbound::Pong)),
            Ok(Message::Close(frame)) => Some(Ok(frame.map_or(
                Inbound::Close {
                    code: None,
                    reason: String::new(),
                },
                |frame| Inbound::Close {
                    code: Some(u16::from(frame.code)),
                    reason: frame.reason.to_string(),
                },
            ))),
            Ok(Message::Frame(_)) => Some(Err(WsError::Transport(
                "unexpected raw WebSocket frame".to_owned(),
            ))),
            Err(error) => Some(Err(WsError::Transport(error.to_string()))),
        }
    }
}
