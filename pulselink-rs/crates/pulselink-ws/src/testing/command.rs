//! Mock relay commands and server state.

use std::collections::HashMap;

use pulselink_protocol::socket::v4::{AppMessage, DeviceOperate, OperationEndReason, RemoteDevice};

pub(super) enum RelayCommand {
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

pub(super) struct ServerState {
    pub(super) clients: HashMap<String, Vec<RemoteDevice>>,
    pub(super) active_ops: Vec<(String, String, DeviceOperate)>,
    pub(super) defer_operate: bool,
    pub(super) no_response: bool,
    pub(super) rpc_error: Option<String>,
}
