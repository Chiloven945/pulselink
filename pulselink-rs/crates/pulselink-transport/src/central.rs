//! Device discovery, connection, and reconnection traits.

use std::time::Duration;

use async_trait::async_trait;

use crate::error::TransportError;
use crate::link::{DiscoveredDevice, GattLink, ScanFilter, TransportDeviceId};

/// The target used for reconnection.
#[derive(Clone, Debug)]
pub struct ReconnectTarget {
    /// Device id captured at the original connection.
    pub device_id: TransportDeviceId,
    /// Filter used to rediscover the device if the id is stale.
    pub filter: ScanFilter,
}

/// A source of devices and links.
#[async_trait]
pub trait Central: Send + Sync {
    /// Scans for devices for up to `timeout`.
    ///
    /// Implementations should return devices discovered so far when the timeout
    /// expires. The future must stop the underlying scan when dropped.
    async fn scan(
        &self,
        filter: ScanFilter,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, TransportError>;

    /// Connects to a discovered device.
    ///
    /// The future must cancel the connection attempt when dropped.
    async fn connect(&self, device: &DiscoveredDevice)
    -> Result<Box<dyn GattLink>, TransportError>;
}

/// Re-establishes a previously known link.
#[async_trait]
pub trait Connector: Send + Sync {
    /// Reconnects to the target, rescanning if necessary.
    async fn reconnect(
        &self,
        target: &ReconnectTarget,
    ) -> Result<Box<dyn GattLink>, TransportError>;
}

/// A complete local-device backend.
pub trait Backend: Central + Connector {}

impl<T: Central + Connector> Backend for T {}
