//! Frame and response helpers for the mock relay.

use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use pulselink_protocol::socket::v4::{
    AppMessage, DeviceOperate, DeviceOperateResult, OperationEndReason, RelayFrame, RpcResponse,
};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use crate::error::WsError;

pub(super) type ServerSocket = WebSocketStream<TcpStream>;
pub(super) type ServerSink = SplitSink<ServerSocket, Message>;

pub(super) async fn send_error(
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

pub(super) async fn send_operation_result(
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

pub(super) async fn send_app(
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

pub(super) async fn send_frame(sink: &mut ServerSink, frame: RelayFrame) -> Result<(), WsError> {
    let text = serde_json::to_string(&frame).map_err(WsError::from)?;
    sink.send(Message::Text(text.into()))
        .await
        .map_err(|error| WsError::Transport(error.to_string()))
}
