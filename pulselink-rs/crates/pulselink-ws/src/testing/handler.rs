//! Mock relay command and controller-frame handlers.

use futures_util::SinkExt;
use pulselink_protocol::socket::v4::{
    AppEvent, AppMessage, ClearOperate, DeviceOperate, OperationEndReason, RelayFrame, RpcResponse,
};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

use crate::error::WsError;

use super::command::{RelayCommand, ServerState};
use super::frames::{ServerSink, send_app, send_error, send_frame, send_operation_result};

pub(super) async fn handle_command(
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
pub(super) async fn handle_controller_frame(
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

pub(super) fn clear_matches(clear: Option<&ClearOperate>, operate: &DeviceOperate) -> bool {
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
