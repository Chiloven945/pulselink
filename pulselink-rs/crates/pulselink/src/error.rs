//! Facade error type.

/// An error returned by the `pulselink` facade.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Core domain validation failed.
    #[error(transparent)]
    Core(#[from] pulselink_core::CoreError),
    /// Protocol encoding or decoding failed.
    #[error(transparent)]
    Protocol(#[from] pulselink_protocol::ProtocolError),
    /// A local transport operation failed.
    #[error(transparent)]
    Transport(#[from] pulselink_transport::TransportError),
    /// A BLE backend operation failed.
    #[cfg(feature = "ble")]
    #[error(transparent)]
    Ble(#[from] pulselink_ble::BleError),
    /// A Coyote V3 runtime operation failed.
    #[cfg(feature = "ble")]
    #[error(transparent)]
    Runtime(#[from] pulselink_runtime::RuntimeError),
    /// A V4 WebSocket operation failed.
    #[cfg(feature = "websocket")]
    #[error(transparent)]
    Ws(#[from] pulselink_ws::WsError),
}
