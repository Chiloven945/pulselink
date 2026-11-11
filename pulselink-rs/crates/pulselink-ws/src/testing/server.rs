//! Mock relay accept and select loop.

use futures_util::StreamExt;
use std::collections::HashMap;

use pulselink_protocol::socket::v4::RelayFrame;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use super::command::{RelayCommand, ServerState};
use super::frames::send_frame;
use super::handler::{handle_command, handle_controller_frame};

pub(super) async fn server(
    listener: TcpListener,
    mut commands: mpsc::Receiver<RelayCommand>,
    received: mpsc::Sender<RelayFrame>,
) {
    let Ok((stream, _)) = listener.accept().await else {
        return;
    };
    let Ok(websocket) = accept_async(stream).await else {
        return;
    };
    let (mut sink, mut source) = websocket.split();
    if send_frame(
        &mut sink,
        RelayFrame::Hello {
            client_id: "controller-1".to_owned(),
        },
    )
    .await
    .is_err()
    {
        return;
    }

    let mut state = ServerState {
        clients: HashMap::new(),
        active_ops: Vec::new(),
        defer_operate: false,
        no_response: false,
        rpc_error: None,
    };

    loop {
        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else { break; };
                if handle_command(&mut sink, &mut state, command).await.is_err() {
                    break;
                }
            }
            message = source.next() => {
                let Some(message) = message else { break; };
                let Ok(message) = message else { break; };
                match message {
                    Message::Text(text) => {
                        let Ok(frame) = serde_json::from_str::<RelayFrame>(&text) else { continue; };
                        let _ = received.send(frame.clone()).await;
                        if handle_controller_frame(&mut sink, &mut state, frame).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => {}
                }
            }
        }
    }
}
