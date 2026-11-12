//! Device scanning, classification, and local-session creation.

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::DeviceKind;
use pulselink_runtime::{Client as RuntimeClient, CoyoteV3Config};
use pulselink_transport::{Backend, DiscoveredDevice, ScanFilter};
use tracing::debug;

use crate::civet::CivetSession;
use crate::coyote_v2::CoyoteV2Session;
use crate::error::DeviceError;
use crate::opossum::OpossumSession;
use crate::paw_prints::PawPrintsSession;
use crate::profile;
use crate::session::LocalSession;

/// A client that discovers and connects local DG-LAB devices.
pub struct DeviceClient {
    backend: Arc<dyn Backend>,
}

impl DeviceClient {
    /// Creates a client from a transport backend.
    #[must_use]
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self { backend }
    }

    /// Scans for devices.
    pub async fn scan(
        &self,
        filter: ScanFilter,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, DeviceError> {
        self.backend
            .scan(filter, timeout)
            .await
            .map_err(DeviceError::from)
    }

    /// Scans using a built-in profile.
    ///
    /// Name advertisements are tried first. If none match, the scan retries
    /// with the profile service UUIDs so firmware privacy-broadcast devices
    /// that omit `LocalName` can still be discovered.
    pub async fn scan_profile(
        &self,
        profile: &profile::DeviceProfile,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, DeviceError> {
        let named = self.scan(profile.scan_filter(), timeout).await?;
        let named: Vec<_> = named
            .into_iter()
            .filter(|device| profile.matches(device))
            .collect();
        if !named.is_empty() {
            return Ok(named);
        }

        debug!(
            kind = ?profile.kind,
            "no named devices; retrying with service-UUID fallback"
        );
        let fallback = self.scan(profile.fallback_scan_filter(), timeout).await?;
        Ok(fallback
            .into_iter()
            .filter(|device| profile.matches(device))
            .collect())
    }

    /// Connects to a device, dispatching by its discovered profile.
    pub async fn connect(&self, device: &DiscoveredDevice) -> Result<LocalSession, DeviceError> {
        let kind = profile::classify(device);
        match kind {
            DeviceKind::CoyoteV3 => self.connect_coyote_v3(device).await,
            DeviceKind::CoyoteV2 => {
                let link = self.backend.connect(device).await?;
                Ok(LocalSession::CoyoteV2(CoyoteV2Session::start(link).await?))
            }
            DeviceKind::Opossum => {
                let link = self.backend.connect(device).await?;
                Ok(LocalSession::Opossum(OpossumSession::start(link).await?))
            }
            DeviceKind::Civet => {
                let link = self.backend.connect(device).await?;
                Ok(LocalSession::Civet(CivetSession::start(link).await?))
            }
            DeviceKind::PawPrints => {
                let link = self.backend.connect(device).await?;
                Ok(LocalSession::PawPrints(
                    PawPrintsSession::start(link).await?,
                ))
            }
            other => Err(DeviceError::Unsupported(other)),
        }
    }

    /// Connects and initializes a Coyote V3 session with default configuration.
    pub async fn connect_coyote_v3(
        &self,
        device: &DiscoveredDevice,
    ) -> Result<LocalSession, DeviceError> {
        self.connect_coyote_v3_with(device, CoyoteV3Config::default())
            .await
    }

    /// Connects and initializes a Coyote V3 session with explicit configuration.
    pub async fn connect_coyote_v3_with(
        &self,
        device: &DiscoveredDevice,
        config: CoyoteV3Config,
    ) -> Result<LocalSession, DeviceError> {
        let client = RuntimeClient::new(Arc::clone(&self.backend));
        let session = client.connect_coyote_v3(device, config).await?;
        Ok(LocalSession::CoyoteV3(session))
    }
}
