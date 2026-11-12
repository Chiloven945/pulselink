//! Local device link types and traits.

use std::fmt;
use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use uuid::Uuid;

use crate::error::TransportError;

/// A stream of characteristic notifications.
pub type NotificationStream =
    Pin<Box<dyn Stream<Item = Result<Vec<u8>, TransportError>> + Send + 'static>>;

/// An opaque platform-specific device identifier.
///
/// The value is not guaranteed to be stable across restarts or operating
/// systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TransportDeviceId(String);

impl TransportDeviceId {
    /// Creates a transport device id from an opaque value.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the opaque id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TransportDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for TransportDeviceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TransportDeviceId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A device discovered during a scan.
#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    /// Opaque device id.
    pub id: TransportDeviceId,
    /// Local name, when advertised.
    pub name: Option<String>,
    /// Received signal strength, when available.
    pub rssi: Option<i16>,
    /// Advertised service UUIDs.
    pub service_uuids: Vec<Uuid>,
    /// Manufacturer-specific data.
    pub manufacturer_data: Vec<(u16, Vec<u8>)>,
}

/// A filter applied during scanning and reconnection.
#[derive(Clone, Debug, Default)]
pub struct ScanFilter {
    /// Match devices whose local name starts with any of these prefixes.
    pub name_prefixes: Vec<String>,
    /// Match devices advertising any of these service UUIDs.
    pub service_uuids: Vec<Uuid>,
}

impl ScanFilter {
    /// Returns whether a discovered device matches this filter.
    ///
    /// An empty filter matches every device. Otherwise, a name-prefix match or a
    /// service-UUID match is sufficient (OR semantics).
    #[must_use]
    pub fn matches(&self, device: &DiscoveredDevice) -> bool {
        if self.name_prefixes.is_empty() && self.service_uuids.is_empty() {
            return true;
        }

        let name_matches = device.name.as_deref().is_some_and(|name| {
            self.name_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
        });
        let service_matches = self
            .service_uuids
            .iter()
            .any(|uuid| device.service_uuids.contains(uuid));

        name_matches || service_matches
    }
}

/// The write mode used for a GATT characteristic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    /// Let the backend choose based on characteristic properties.
    Auto,
    /// Require a write response.
    WithResponse,
    /// Use a write command without a response.
    WithoutResponse,
}

/// A bidirectional local device link.
#[async_trait]
pub trait GattLink: Send + Sync {
    /// Returns the device id for this link.
    fn device_id(&self) -> &TransportDeviceId;

    /// Returns whether the link is currently believed to be connected.
    fn is_connected(&self) -> bool;

    /// Returns the negotiated ATT MTU, when the backend exposes it.
    fn mtu(&self) -> Option<u16> {
        None
    }

    /// Writes bytes to a characteristic.
    async fn write(
        &self,
        characteristic: Uuid,
        data: &[u8],
        mode: WriteMode,
    ) -> Result<(), TransportError>;

    /// Reads bytes from a characteristic.
    async fn read(&self, characteristic: Uuid) -> Result<Vec<u8>, TransportError>;

    /// Subscribes to notifications for a characteristic.
    async fn subscribe(&self, characteristic: Uuid) -> Result<NotificationStream, TransportError>;

    /// Disconnects the link. Implementations must make this idempotent.
    async fn disconnect(&self) -> Result<(), TransportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(name: Option<&str>, services: &[Uuid]) -> DiscoveredDevice {
        DiscoveredDevice {
            id: TransportDeviceId::new("dev"),
            name: name.map(ToOwned::to_owned),
            rssi: Some(-40),
            service_uuids: services.to_vec(),
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn empty_filter_matches_everything() {
        let filter = ScanFilter::default();
        assert!(filter.matches(&device(Some("Coyote"), &[])));
    }

    #[test]
    fn name_prefix_matches() {
        let filter = ScanFilter {
            name_prefixes: vec!["47L".to_owned()],
            ..ScanFilter::default()
        };
        assert!(filter.matches(&device(Some("47L121000"), &[])));
        assert!(!filter.matches(&device(Some("Other"), &[])));
    }

    #[test]
    fn service_uuid_matches() {
        let uuid = Uuid::from_u128(0x180c);
        let filter = ScanFilter {
            service_uuids: vec![uuid],
            ..ScanFilter::default()
        };
        assert!(filter.matches(&device(None, &[uuid])));
        assert!(!filter.matches(&device(None, &[])));
    }

    #[test]
    fn name_or_service_matches() {
        let uuid = Uuid::from_u128(0x180c);
        let filter = ScanFilter {
            name_prefixes: vec!["47L".to_owned()],
            service_uuids: vec![uuid],
        };
        assert!(filter.matches(&device(Some("47L121000"), &[])));
        assert!(filter.matches(&device(None, &[uuid])));
        assert!(!filter.matches(&device(Some("Other"), &[])));
    }

    #[test]
    fn device_id_display_and_conversions() {
        let id = TransportDeviceId::from("dev-1");
        assert_eq!(id.to_string(), "dev-1");
        assert_eq!(id.as_str(), "dev-1");
        assert_eq!(TransportDeviceId::from(String::from("x")).as_str(), "x");
    }
}
