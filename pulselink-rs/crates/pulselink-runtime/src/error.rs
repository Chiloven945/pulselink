//! Runtime errors.

use pulselink_core::{CoreError, DeviceKind};
use pulselink_protocol::ProtocolError;
use pulselink_protocol::coyote::v3::Sequence;
use pulselink_transport::TransportError;

/// An error returned by the device session runtime.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RuntimeError {
    /// The underlying transport failed.
    #[error(transparent)]
    Transport(#[from] TransportError),
    /// Protocol encoding or decoding failed.
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    /// Core domain validation failed.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// The session is not ready for control operations.
    #[error("session is not ready")]
    NotReady,
    /// The device kind is not supported by this session.
    #[error("unsupported device: {0:?}")]
    Unsupported(DeviceKind),
    /// The B1 acknowledgement did not arrive in time.
    #[error("ack timeout for sequence {0:?}")]
    AckTimeout(Sequence),
    /// The operation was cancelled.
    #[error("operation cancelled")]
    Cancelled,
    /// The session is closed.
    #[error("session closed")]
    Closed,
    /// A safety policy rejected the operation.
    #[error("safety limit exceeded: {0}")]
    SafetyLimit(String),
    /// The command queue is full.
    #[error("command queue is full")]
    Busy,
    /// Reconnection failed after the configured number of attempts.
    #[error("reconnect failed after {0} attempts")]
    ReconnectFailed(u32),
}
