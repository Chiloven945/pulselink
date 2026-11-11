//! Errors returned by the V4 WebSocket SDK.

/// An error returned by the V4 socket, RPC layer, or device operations.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WsError {
    /// The underlying WebSocket transport failed.
    #[error("websocket error: {0}")]
    Transport(String),
    /// JSON encoding or decoding failed.
    #[error("JSON error: {0}")]
    Json(String),
    /// An app returned an RPC error string.
    #[error("RPC error: {code}")]
    Rpc {
        /// Error code or reason returned by the app.
        code: String,
        /// Method that produced the error.
        method: String,
    },
    /// An RPC response did not arrive in time.
    #[error("RPC timeout for {req_id}")]
    RpcTimeout {
        /// Request id that timed out.
        req_id: String,
    },
    /// A client id is not registered on this socket.
    #[error("client not found: {0}")]
    ClientNotFound(String),
    /// A device slot id is not present in the cache.
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    /// The socket is disconnected.
    #[error("not connected")]
    Disconnected,
    /// The socket has not received a `hello` frame.
    #[error("socket is not paired")]
    NotPaired,
    /// The app reported a duplicate request id.
    #[error("duplicate request id: {0}")]
    DuplicateRequestId(String),
    /// Parameters or protocol validation failed.
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    /// The operation was cancelled.
    #[error("operation cancelled")]
    Cancelled,
    /// The socket was closed locally.
    #[error("socket closed")]
    Closed,
}

impl From<serde_json::Error> for WsError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

impl WsError {
    pub(crate) fn from_rpc(code: String, method: String) -> Self {
        match code.as_str() {
            "duplicate_request_id" => Self::DuplicateRequestId(method),
            "invalid_params" | "invalid_operate" => Self::InvalidParams(code),
            "slot_not_found" => Self::DeviceNotFound(code),
            _ => Self::Rpc { code, method },
        }
    }
}
