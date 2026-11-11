//! Errors produced by the BLE backend.

use std::error::Error;

/// An error produced by the btleplug BLE backend.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BleError {
    /// No Bluetooth adapter is available.
    #[error("no Bluetooth adapter available")]
    NoAdapter,
    /// Scanning failed.
    #[error("scan failed: {0}")]
    Scan(String),
    /// The requested device was not found.
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    /// Connection failed.
    #[error("connection failed: {0}")]
    Connect(String),
    /// A GATT operation failed.
    #[error("GATT operation failed: {0}")]
    Gatt(String),
    /// A notification operation failed.
    #[error("notification operation failed: {0}")]
    Notify(String),
    /// An operation timed out.
    #[error("operation timed out")]
    Timeout,
    /// The link is disconnected.
    #[error("link disconnected")]
    Disconnected,
    /// A backend-specific error.
    #[error(transparent)]
    Backend(#[from] Box<dyn Error + Send + Sync>),
}

#[cfg(feature = "btleplug")]
impl BleError {
    pub(crate) fn from_btleplug(error: btleplug::Error) -> Self {
        match error {
            btleplug::Error::NoAdapterAvailable => Self::NoAdapter,
            btleplug::Error::DeviceNotFound => Self::DeviceNotFound("unknown".to_owned()),
            btleplug::Error::TimedOut(_) => Self::Timeout,
            btleplug::Error::NotConnected => Self::Disconnected,
            other => Self::Backend(Box::new(other)),
        }
    }
}

#[cfg(all(test, feature = "btleplug"))]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn maps_specific_btleplug_errors() {
        assert!(matches!(
            BleError::from_btleplug(btleplug::Error::NoAdapterAvailable),
            BleError::NoAdapter
        ));
        assert!(matches!(
            BleError::from_btleplug(btleplug::Error::TimedOut(Duration::from_secs(1))),
            BleError::Timeout
        ));
        assert!(matches!(
            BleError::from_btleplug(btleplug::Error::NotConnected),
            BleError::Disconnected
        ));
        assert!(matches!(
            BleError::from_btleplug(btleplug::Error::DeviceNotFound),
            BleError::DeviceNotFound(_)
        ));
    }

    #[test]
    fn preserves_other_errors_as_backend_errors() {
        assert!(matches!(
            BleError::from_btleplug(btleplug::Error::PermissionDenied),
            BleError::Backend(_)
        ));
    }
}
