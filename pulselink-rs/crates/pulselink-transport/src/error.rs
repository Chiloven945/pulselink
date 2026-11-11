//! Transport errors.

use crate::link::TransportDeviceId;

/// Errors returned by local device transports.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TransportError {
    /// No BLE adapter is available.
    #[error("no BLE adapter available")]
    NoAdapter,
    /// Scanning failed.
    #[error("scan failed: {0}")]
    Scan(String),
    /// The requested device was not found.
    #[error("device not found: {0}")]
    DeviceNotFound(TransportDeviceId),
    /// Connection failed.
    #[error("connection failed: {0}")]
    Connect(String),
    /// A GATT operation failed.
    #[error("GATT operation failed: {0}")]
    Gatt(String),
    /// Notification subscription failed.
    #[error("notification subscription failed: {0}")]
    Notify(String),
    /// The link is disconnected.
    #[error("link disconnected")]
    Disconnected,
    /// The operation timed out.
    #[error("operation timed out")]
    Timeout,
    /// An opaque backend error.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync>),
}
