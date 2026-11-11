//! Device identity types.

use std::fmt;

/// An opaque BLE peripheral identifier.
///
/// The value is platform-specific and is not guaranteed to be stable across
/// restarts or operating systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BleDeviceId(String);

impl BleDeviceId {
    /// Creates a BLE device identifier from an opaque platform value.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the opaque identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BleDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifies a device exposed by a DG-LAB V4 app client.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemoteDeviceId {
    /// The app client that exposes the device.
    pub client_id: String,
    /// The device slot inside that app client.
    pub slot_id: String,
}

impl RemoteDeviceId {
    /// Creates a remote device identifier.
    #[must_use]
    pub fn new(client_id: impl Into<String>, slot_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            slot_id: slot_id.into(),
        }
    }
}

impl fmt::Display for RemoteDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.client_id, self.slot_id)
    }
}

/// Identifies a device known to `pulselink-rs`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DeviceId {
    /// A device reached directly over BLE.
    Ble(BleDeviceId),
    /// A device exposed by a DG-LAB V4 app client.
    Remote(RemoteDeviceId),
}

impl DeviceId {
    /// Creates a BLE device identifier.
    #[must_use]
    pub fn ble(id: impl Into<String>) -> Self {
        Self::Ble(BleDeviceId::new(id))
    }

    /// Creates a remote V4 device identifier.
    #[must_use]
    pub fn remote(client_id: impl Into<String>, slot_id: impl Into<String>) -> Self {
        Self::Remote(RemoteDeviceId::new(client_id, slot_id))
    }

    /// Returns the BLE identifier, if this is a BLE device.
    #[must_use]
    pub const fn as_ble(&self) -> Option<&BleDeviceId> {
        match self {
            Self::Ble(id) => Some(id),
            Self::Remote(_) => None,
        }
    }

    /// Returns the remote identifier, if this is a remote device.
    #[must_use]
    pub const fn as_remote(&self) -> Option<&RemoteDeviceId> {
        match self {
            Self::Ble(_) => None,
            Self::Remote(id) => Some(id),
        }
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ble(id) => write!(f, "ble:{id}"),
            Self::Remote(id) => write!(f, "remote:{id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_ids_display_and_roundtrip() {
        let ble = DeviceId::ble("peripheral-1");
        assert_eq!(ble.to_string(), "ble:peripheral-1");
        assert!(ble.as_ble().is_some());
        assert!(ble.as_remote().is_none());

        let remote = DeviceId::remote("client-1", "slot-a");
        assert_eq!(remote.to_string(), "remote:client-1:slot-a");
        assert!(remote.as_remote().is_some());
        assert!(remote.as_ble().is_none());
    }
}
