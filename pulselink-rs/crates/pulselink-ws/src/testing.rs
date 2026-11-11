//! A local WebSocket relay for integration tests.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use pulselink_protocol::socket::v4::{
    AppEvent, AppMessage, ClearOperate, DeviceOperate, DeviceOperateResult, OperationEndReason,
    RelayFrame, RemoteDevice, RpcResponse,
};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, accept_async};

use crate::error::WsError;

type ServerSocket = WebSocketStream<TcpStream>;
type ServerSink = SplitSink<ServerSocket, Message>;

/// A scriptable local relay used by `pulselink-ws` tests.
#[derive(Clone)]
pub struct MockRelay {
    url: String,
    commands: mpsc::Sender<RelayCommand>,
    received: Arc<Mutex<mpsc::Receiver<RelayFrame>>>,
}

impl MockRelay {
    /// Starts a relay on an ephemeral localhost port.
    pub async fn start() -> Result<Self, WsError> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| WsError::Transport(error.to_string()))?;
        let address = listener
            .local_addr()
            .map_err(|error| WsError::Transport(error.to_string()))?;
        let (commands, command_rx) = mpsc::channel(64);
        let (received_tx, received_rx) = mpsc::channel(64);
        tokio::spawn(server(listener, command_rx, received_tx));
        Ok(Self {
            url: format!("ws://{address}"),
            commands,
            received: Arc::new(Mutex::new(received_rx)),
        })
    }

    /// Returns the relay's WebSocket URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Attaches an app client and sends its initial device snapshot.
    pub async fn attach_client(&self, client_id: &str, devices: Vec<RemoteDevice>) {
        let _ = self
            .commands
            .send(RelayCommand::AttachClient {
                client_id: client_id.to_owned(),
                devices,
            })
            .await;
    }

    /// Detaches an app client.
    pub async fn detach_client(&self, client_id: &str) {
        let _ = self
            .commands
            .send(RelayCommand::DetachClient {
                client_id: client_id.to_owned(),
            })
            .await;
    }

    /// Sends an app message to the controller.
    pub async fn send_app_message(&self, client_id: &str, message: AppMessage) {
        let _ = self
            .commands
            .send(RelayCommand::SendAppMessage {
                client_id: client_id.to_owned(),
                message,
            })
            .await;
    }

    /// Sends a server heartbeat.
    pub async fn heartbeat(&self) {
        let _ = self.commands.send(RelayCommand::Heartbeat).await;
    }

    /// Sends a server-level pong.
    pub async fn pong(&self, ts: i64) {
        let _ = self.commands.send(RelayCommand::Pong { ts }).await;
    }

    /// Sends an idle-timeout frame.
    pub async fn idle_timeout(&self) {
        let _ = self.commands.send(RelayCommand::IdleTimeout).await;
    }

    /// Sends a WebSocket close frame.
    pub async fn close(&self, code: u16, reason: &str) {
        let _ = self
            .commands
            .send(RelayCommand::Close {
                code,
                reason: reason.to_owned(),
            })
            .await;
    }

    /// Defers `device.op` responses until a clear or explicit completion.
    pub async fn set_defer_operate(&self, defer: bool) {
        let _ = self
            .commands
            .send(RelayCommand::SetDeferOperate { defer })
            .await;
    }

    /// Stops responding to RPC requests.
    pub async fn set_no_response(&self, no_response: bool) {
        let _ = self
            .commands
            .send(RelayCommand::SetNoResponse { no_response })
            .await;
    }

    /// Makes every RPC return the given error string.
    pub async fn set_rpc_error(&self, error: Option<String>) {
        let _ = self
            .commands
            .send(RelayCommand::SetRpcError { error })
            .await;
    }

    /// Completes a deferred operation manually.
    pub async fn complete_operation(&self, req_id: &str, reason: OperationEndReason) {
        let _ = self
            .commands
            .send(RelayCommand::CompleteOperation {
                req_id: req_id.to_owned(),
                reason,
            })
            .await;
    }

    /// Receives the next relay frame sent by the controller.
    pub async fn recv(&self) -> Option<RelayFrame> {
        self.received.lock().await.recv().await
    }
}

enum RelayCommand {
    AttachClient {
        client_id: String,
        devices: Vec<RemoteDevice>,
    },
    DetachClient {
        client_id: String,
    },
    SendAppMessage {
        client_id: String,
        message: AppMessage,
    },
    Heartbeat,
    Pong {
        ts: i64,
    },
    IdleTimeout,
    Close {
        code: u16,
        reason: String,
    },
    SetDeferOperate {
        defer: bool,
    },
    SetNoResponse {
        no_response: bool,
    },
    SetRpcError {
        error: Option<String>,
    },
    CompleteOperation {
        req_id: String,
        reason: OperationEndReason,
    },
}

struct ServerState {
    clients: HashMap<String, Vec<RemoteDevice>>,
    active_ops: Vec<(String, String, DeviceOperate)>,
    defer_operate: bool,
    no_response: bool,
    rpc_error: Option<String>,
}

async fn server(
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

async fn handle_command(
    sink: &mut ServerSink,
    state: &mut ServerState,
    command: RelayCommand,
) -> Result<(), WsError> {
    match command {
        RelayCommand::AttachClient { client_id, devices } => {
            state.clients.insert(client_id.clone(), devices.clone());
            send_frame(
                sink,
                RelayFrame::ClientAttached {
                    client_id: client_id.clone(),
                },
            )
            .await?;
            send_app(
                sink,
                &client_id,
                AppMessage::Ev(AppEvent::DevicesSnapshot { devices }),
            )
            .await?;
        }
        RelayCommand::DetachClient { client_id } => {
            state.clients.remove(&client_id);
            send_frame(sink, RelayFrame::ClientDisconnected { client_id }).await?;
        }
        RelayCommand::SendAppMessage { client_id, message } => {
            send_app(sink, &client_id, message).await?;
        }
        RelayCommand::Heartbeat => send_frame(sink, RelayFrame::Heartbeat).await?,
        RelayCommand::Pong { ts } => send_frame(sink, RelayFrame::Pong { ts: Some(ts) }).await?,
        RelayCommand::IdleTimeout => send_frame(sink, RelayFrame::IdleTimeout).await?,
        RelayCommand::Close { code, reason } => {
            let _ = sink
                .send(Message::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(code),
                    reason: reason.into(),
                })))
                .await;
        }
        RelayCommand::SetDeferOperate { defer } => state.defer_operate = defer,
        RelayCommand::SetNoResponse { no_response } => state.no_response = no_response,
        RelayCommand::SetRpcError { error } => state.rpc_error = error,
        RelayCommand::CompleteOperation { req_id, reason } => {
            if let Some(index) = state.active_ops.iter().position(|(_, id, _)| id == &req_id) {
                let (client_id, _, operate) = state.active_ops.remove(index);
                send_operation_result(sink, &client_id, &req_id, operate, reason).await?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn handle_controller_frame(
    sink: &mut ServerSink,
    state: &mut ServerState,
    frame: RelayFrame,
) -> Result<(), WsError> {
    let RelayFrame::Message { client_id, data } = frame else {
        if matches!(frame, RelayFrame::Ping) {
            send_frame(
                sink,
                RelayFrame::Pong {
                    ts: Some(123_456_789),
                },
            )
            .await?;
        }
        return Ok(());
    };
    let Ok(message) = serde_json::from_value::<AppMessage>(data) else {
        return Ok(());
    };
    let AppMessage::Req(request) = message else {
        return Ok(());
    };
    let client_id = client_id.unwrap_or_else(|| "app-1".to_owned());

    if state.no_response {
        return Ok(());
    }
    if let Some(error) = &state.rpc_error {
        send_app(
            sink,
            &client_id,
            AppMessage::Resp(RpcResponse {
                req_id: request.req_id,
                result: None,
                error: Some(error.clone()),
            }),
        )
        .await?;
        return Ok(());
    }

    match request.m.as_str() {
        "devices.get" => {
            let devices = state.clients.get(&client_id).cloned().unwrap_or_default();
            send_app(
                sink,
                &client_id,
                AppMessage::Resp(RpcResponse {
                    req_id: request.req_id,
                    result: Some(json!({ "devices": devices })),
                    error: None,
                }),
            )
            .await?;
        }
        "ping" => {
            send_app(
                sink,
                &client_id,
                AppMessage::Resp(RpcResponse {
                    req_id: request.req_id,
                    result: Some(json!(123_456_789)),
                    error: None,
                }),
            )
            .await?;
        }
        "device.op" => {
            let Ok(operate) =
                serde_json::from_value::<DeviceOperate>(request.data.unwrap_or(Value::Null))
            else {
                send_error(sink, &client_id, request.req_id, "invalid_params").await?;
                return Ok(());
            };
            if state.defer_operate {
                state.active_ops.push((client_id, request.req_id, operate));
            } else {
                send_operation_result(
                    sink,
                    &client_id,
                    &request.req_id,
                    operate,
                    OperationEndReason::Completed,
                )
                .await?;
            }
        }
        "device.op.clear" => {
            let clear: Option<ClearOperate> = request
                .data
                .and_then(|data| serde_json::from_value(data).ok());
            let mut remaining = Vec::new();
            for (op_client_id, req_id, operate) in state.active_ops.drain(..) {
                if clear_matches(clear.as_ref(), &operate) {
                    send_operation_result(
                        sink,
                        &op_client_id,
                        &req_id,
                        operate,
                        OperationEndReason::Cleared,
                    )
                    .await?;
                } else {
                    remaining.push((op_client_id, req_id, operate));
                }
            }
            state.active_ops = remaining;
            send_app(
                sink,
                &client_id,
                AppMessage::Resp(RpcResponse {
                    req_id: request.req_id,
                    result: Some(json!({})),
                    error: None,
                }),
            )
            .await?;
        }
        _ => {
            send_error(sink, &client_id, request.req_id, "method_not_found").await?;
        }
    }
    Ok(())
}

fn clear_matches(clear: Option<&ClearOperate>, operate: &DeviceOperate) -> bool {
    let Some(clear) = clear else {
        return true;
    };
    if let Some(slot_id) = &clear.slot_id
        && slot_id != &operate.slot_id
    {
        return false;
    }
    if let Some(channel) = clear.channel
        && channel != operate.channel
    {
        return false;
    }
    true
}

async fn send_error(
    sink: &mut ServerSink,
    client_id: &str,
    req_id: String,
    error: &str,
) -> Result<(), WsError> {
    send_app(
        sink,
        client_id,
        AppMessage::Resp(RpcResponse {
            req_id,
            result: None,
            error: Some(error.to_owned()),
        }),
    )
    .await
}

async fn send_operation_result(
    sink: &mut ServerSink,
    client_id: &str,
    req_id: &str,
    operate: DeviceOperate,
    reason: OperationEndReason,
) -> Result<(), WsError> {
    let result = DeviceOperateResult {
        action: operate.action,
        reason,
        slot_id: Some(operate.slot_id.clone()),
        channel: Some(operate.channel),
    };
    send_app(
        sink,
        client_id,
        AppMessage::Resp(RpcResponse {
            req_id: req_id.to_owned(),
            result: Some(serde_json::to_value(result).map_err(WsError::from)?),
            error: None,
        }),
    )
    .await
}

async fn send_app(
    sink: &mut ServerSink,
    client_id: &str,
    message: AppMessage,
) -> Result<(), WsError> {
    send_frame(
        sink,
        RelayFrame::Message {
            client_id: Some(client_id.to_owned()),
            data: serde_json::to_value(message).map_err(WsError::from)?,
        },
    )
    .await
}

async fn send_frame(sink: &mut ServerSink, frame: RelayFrame) -> Result<(), WsError> {
    let text = serde_json::to_string(&frame).map_err(WsError::from)?;
    sink.send(Message::Text(text.into()))
        .await
        .map_err(|error| WsError::Transport(error.to_string()))
}
