//! Errors returned by local device drivers.

use pulselink_core::{CoreError, DeviceKind};
use pulselink_protocol::ProtocolError;
use pulselink_runtime::RuntimeError;
use pulselink_transport::TransportError;

/// An error returned while connecting to or controlling a local device.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DeviceError {
    /// The transport failed.
    #[error(transparent)]
    Transport(#[from] TransportError),
    /// A protocol codec failed.
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    /// A core value was invalid.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// The Coyote V3 runtime failed.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    /// The device kind is not implemented.
    #[error("unsupported device: {0:?}")]
    Unsupported(DeviceKind),
    /// The device did not match a known profile.
    #[error("unknown device profile: {0}")]
    UnknownProfile(String),
    /// The session is closed.
    #[error("device session is closed")]
    Closed,
    /// A notification was malformed.
    #[error("invalid device notification: {0}")]
    InvalidNotification(String),
}
