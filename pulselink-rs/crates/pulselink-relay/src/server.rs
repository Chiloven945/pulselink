//! TCP/WebSocket server and per-connection forwarding.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::SinkExt;
use futures_util::StreamExt;
use futures_util::stream::{SplitSink, SplitStream};
use pulselink_protocol::socket::v4::RelayFrame;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::registry::{AttachError, RelayState, ServerCommand, send};

type Socket = WebSocketStream<TcpStream>;
type SocketSink = SplitSink<Socket, Message>;
type SocketSource = SplitStream<Socket>;

pub(crate) async fn run(
    listener: TcpListener,
    state: Arc<RelayState>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            accepted = listener.accept() => {
                match accepted {
                    Ok((stream, peer)) => {
                        let state = Arc::clone(&state);
                        tokio::spawn(async move {
                            if let Err(error) = handle_socket(stream, state).await {
                                debug!(%peer, %error, "relay connection ended");
                            }
                        });
                    }
                    Err(error) => warn!(%error, "relay accept failed"),
                }
            }
        }
    }
}

#[allow(clippy::result_large_err)]
async fn handle_socket(stream: TcpStream, state: Arc<RelayState>) -> Result<(), String> {
    let mut query = None;
    let callback = |request: &Request, response| {
        query = request.uri().query().map(str::to_owned);
        Ok(response)
    };
    let socket = accept_hdr_async(stream, callback)
        .await
        .map_err(|error| error.to_string())?;
    let target = query.as_deref().and_then(parse_target_id);

    match target {
        Some(target_id) => handle_app(socket, state, target_id).await,
        None => handle_controller(socket, state).await,
    }
}

fn parse_target_id(query: &str) -> Option<String> {
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "tid")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
}

async fn handle_controller(socket: Socket, state: Arc<RelayState>) -> Result<(), String> {
    let (sink, source) = socket.split();
    let (outbound, receiver) = mpsc::channel(64);
    let writer = tokio::spawn(writer_task(sink, receiver));
    let target_id = Uuid::new_v4().simple().to_string();
    state.register_controller(target_id.clone(), outbound.clone());

    send(
        &outbound,
        RelayFrame::Hello {
            client_id: target_id.clone(),
        },
    );
    let heartbeat = spawn_heartbeat(outbound.clone(), state.config.heartbeat_interval);
    let idle = state
        .config
        .idle_timeout
        .map(|timeout| tokio::spawn(idle_task(Arc::clone(&state), target_id.clone(), timeout)));

    let _ = read_controller(source, Arc::clone(&state), target_id.clone()).await;
    heartbeat.abort();
    if let Some(idle) = idle {
        idle.abort();
    }

    if let Some(removal) = state.remove_controller(&target_id) {
        for (client_id, app) in removal.apps {
            send(
                &app,
                RelayFrame::ControllerDisconnected {
                    client_id: target_id.clone(),
                },
            );
            let _ = app.try_send(ServerCommand::Close {
                code: 4000,
                reason: "controller disconnected".to_owned(),
            });
            debug!(%client_id, "app closed after controller disconnect");
        }
    }
    let _ = outbound.try_send(ServerCommand::Close {
        code: 4000,
        reason: "controller disconnected".to_owned(),
    });
    drop(outbound);
    let _ = writer.await;
    Ok(())
}

async fn handle_app(
    socket: Socket,
    state: Arc<RelayState>,
    target_id: String,
) -> Result<(), String> {
    let (sink, source) = socket.split();
    let (outbound, receiver) = mpsc::channel(64);
    let writer = tokio::spawn(writer_task(sink, receiver));

    let client_id = match state.attach_app(&target_id, &outbound) {
        Ok(client_id) => client_id,
        Err(AttachError::ControllerNotFound) => {
            reject(
                &outbound,
                4001,
                "controller not found",
                "controller_not_found",
            );
            drop(outbound);
            let _ = writer.await;
            return Ok(());
        }
        Err(AttachError::TooManyApps) => {
            reject(
                &outbound,
                1013,
                "too many apps for controller",
                "too_many_apps",
            );
            drop(outbound);
            let _ = writer.await;
            return Ok(());
        }
    };

    let heartbeat = spawn_heartbeat(outbound.clone(), state.config.heartbeat_interval);
    let _ = read_app(
        source,
        Arc::clone(&state),
        outbound.clone(),
        target_id.clone(),
        client_id.clone(),
    )
    .await;
    heartbeat.abort();
    state.detach_app(&target_id, &client_id);
    drop(outbound);
    let _ = writer.await;
    Ok(())
}

async fn read_controller(
    mut source: SocketSource,
    state: Arc<RelayState>,
    target_id: String,
) -> Result<(), String> {
    while let Some(message) = source.next().await {
        let message = message.map_err(|error| error.to_string())?;
        let Message::Text(text) = message else {
            if matches!(message, Message::Close(_)) {
                break;
            }
            continue;
        };
        let frame = match serde_json::from_str::<RelayFrame>(&text) {
            Ok(frame) => frame,
            Err(error) => {
                debug!(%error, "invalid controller frame");
                continue;
            }
        };
        match frame {
            RelayFrame::Message {
                client_id: Some(client_id),
                data,
            } => {
                if !state.route_from_controller(&target_id, &client_id, data) {
                    debug!(%client_id, "controller message for unknown app");
                }
            }
            RelayFrame::Message {
                client_id: None, ..
            } => debug!("controller message without clientId"),
            RelayFrame::Ping => {
                if let Some(outbound) = state.controller_outbound(&target_id) {
                    send(&outbound, RelayFrame::Pong { ts: Some(now_ms()) });
                }
            }
            RelayFrame::Unknown
            | RelayFrame::ClientAttached { .. }
            | RelayFrame::ClientDisconnected { .. }
            | RelayFrame::ControllerAttached { .. }
            | RelayFrame::ControllerDisconnected { .. }
            | RelayFrame::Hello { .. }
            | RelayFrame::Heartbeat
            | RelayFrame::Pong { .. }
            | RelayFrame::IdleTimeout
            | RelayFrame::Error { .. } => {}
        }
    }
    Ok(())
}

async fn read_app(
    mut source: SocketSource,
    state: Arc<RelayState>,
    outbound: mpsc::Sender<ServerCommand>,
    target_id: String,
    client_id: String,
) -> Result<(), String> {
    while let Some(message) = source.next().await {
        let message = message.map_err(|error| error.to_string())?;
        let Message::Text(text) = message else {
            if matches!(message, Message::Close(_)) {
                break;
            }
            continue;
        };
        let frame = match serde_json::from_str::<RelayFrame>(&text) {
            Ok(frame) => frame,
            Err(error) => {
                debug!(%error, "invalid app frame");
                continue;
            }
        };
        match frame {
            RelayFrame::Message { data, .. } => {
                if !state.route_from_app(&target_id, &client_id, data) {
                    break;
                }
            }
            RelayFrame::Ping => send(&outbound, RelayFrame::Pong { ts: Some(now_ms()) }),
            RelayFrame::Unknown
            | RelayFrame::Hello { .. }
            | RelayFrame::ClientAttached { .. }
            | RelayFrame::ClientDisconnected { .. }
            | RelayFrame::ControllerAttached { .. }
            | RelayFrame::ControllerDisconnected { .. }
            | RelayFrame::Heartbeat
            | RelayFrame::Pong { .. }
            | RelayFrame::IdleTimeout
            | RelayFrame::Error { .. } => {}
        }
    }
    Ok(())
}

async fn writer_task(mut sink: SocketSink, mut receiver: mpsc::Receiver<ServerCommand>) {
    while let Some(command) = receiver.recv().await {
        match command {
            ServerCommand::Frame(frame) => {
                let Ok(text) = serde_json::to_string(&frame) else {
                    continue;
                };
                if sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            ServerCommand::Close { code, reason } => {
                let frame = CloseFrame {
                    code: CloseCode::from(code),
                    reason: reason.into(),
                };
                let _ = sink.send(Message::Close(Some(frame))).await;
                break;
            }
        }
    }
}

fn spawn_heartbeat(outbound: mpsc::Sender<ServerCommand>, period: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(period);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if outbound
                .try_send(ServerCommand::Frame(RelayFrame::Heartbeat))
                .is_err()
            {
                break;
            }
        }
    })
}

async fn idle_task(state: Arc<RelayState>, target_id: String, timeout: Duration) {
    let mut ticker = interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        let Some(last_app_at) = state.controller_idle_since(&target_id) else {
            break;
        };
        if state.controller_has_apps(&target_id) || last_app_at.elapsed() < timeout {
            continue;
        }
        if let Some(outbound) = state.controller_outbound(&target_id) {
            send(&outbound, RelayFrame::IdleTimeout);
            let _ = outbound.try_send(ServerCommand::Close {
                code: 4002,
                reason: "idle timeout".to_owned(),
            });
        }
        if let Some(removal) = state.remove_controller(&target_id) {
            for (_, app) in removal.apps {
                let _ = app.try_send(ServerCommand::Close {
                    code: 4000,
                    reason: "controller disconnected".to_owned(),
                });
            }
        }
        break;
    }
}

fn reject(outbound: &mpsc::Sender<ServerCommand>, code: u16, reason: &str, error: &str) {
    send(
        outbound,
        RelayFrame::Error {
            code: Some(error.to_owned()),
            message: Some(reason.to_owned()),
        },
    );
    let _ = outbound.try_send(ServerCommand::Close {
        code,
        reason: reason.to_owned(),
    });
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
        })
}
