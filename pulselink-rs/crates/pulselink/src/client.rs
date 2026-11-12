//! High-level BLE control facade.

use std::sync::Arc;
use std::time::Duration;

use pulselink_ble::BtleplugCentral;
use pulselink_device::{DeviceClient, DeviceProfile, LocalSession};
use pulselink_runtime::{Client, ClientConfig, CoyoteV3Config, CoyoteV3Session};
use pulselink_transport::{Backend, DiscoveredDevice, ScanFilter};

use crate::error::Error;

/// The high-level local-device entry point.
///
/// `PulseLink` owns a runtime [`Client`]; it does not contain a second session
/// implementation. It is available with the `ble` feature.
pub struct PulseLink {
    client: Client,
    devices: DeviceClient,
}

impl PulseLink {
    /// Creates a client backed by the first system Bluetooth adapter.
    pub async fn new() -> Result<Self, Error> {
        let backend = BtleplugCentral::new().await?;
        Ok(Self::with_backend(Arc::new(backend)))
    }

    /// Creates a client from an existing transport backend.
    #[must_use]
    pub fn with_backend(backend: Arc<dyn Backend>) -> Self {
        Self {
            client: Client::new(Arc::clone(&backend)),
            devices: DeviceClient::new(backend),
        }
    }

    /// Creates a client with explicit runtime client configuration.
    #[must_use]
    pub fn with_backend_and_config(backend: Arc<dyn Backend>, config: ClientConfig) -> Self {
        Self {
            client: Client::with_config(Arc::clone(&backend), config),
            devices: DeviceClient::new(backend),
        }
    }

    /// Scans for devices.
    pub async fn scan(
        &self,
        filter: ScanFilter,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, Error> {
        self.client.scan(filter, timeout).await.map_err(Into::into)
    }

    /// Scans using a built-in device profile.
    pub async fn scan_profile(
        &self,
        profile: &DeviceProfile,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, Error> {
        self.devices
            .scan_profile(profile, timeout)
            .await
            .map_err(Into::into)
    }

    /// Connects any supported local device and returns its typed session.
    pub async fn connect_device(&self, device: &DiscoveredDevice) -> Result<LocalSession, Error> {
        self.devices.connect(device).await.map_err(Into::into)
    }

    /// Connects a discovered Coyote V3 device with default session settings.
    pub async fn connect_coyote_v3(
        &self,
        device: &DiscoveredDevice,
    ) -> Result<CoyoteV3Session, Error> {
        self.connect_coyote_v3_with(device, CoyoteV3Config::default())
            .await
    }

    /// Connects a discovered Coyote V3 device with explicit session settings.
    pub async fn connect_coyote_v3_with(
        &self,
        device: &DiscoveredDevice,
        config: CoyoteV3Config,
    ) -> Result<CoyoteV3Session, Error> {
        self.client
            .connect_coyote_v3(device, config)
            .await
            .map_err(Into::into)
    }

    /// Shuts down all sessions created by this client.
    pub async fn shutdown(self) {
        self.client.shutdown().await;
    }
}
