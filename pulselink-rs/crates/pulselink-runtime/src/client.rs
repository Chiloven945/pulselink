//! Client factory and backend scanning.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pulselink_core::{DeviceId, DeviceInfo, DeviceKind};
use pulselink_protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink_transport::{Backend, DiscoveredDevice, ReconnectTarget, ScanFilter};

use crate::config::{ClientConfig, CoyoteV3Config};
use crate::error::RuntimeError;
use crate::session::{CoyoteV3Session, SessionStart};

/// A client that scans for and creates device sessions.
pub struct Client {
    backend: Arc<dyn Backend>,
    config: ClientConfig,
    sessions: Mutex<Vec<CoyoteV3Session>>,
    closed: AtomicBool,
}

impl Client {
    /// Creates a client with default configuration.
    #[must_use]
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self::with_config(backend, ClientConfig::default())
    }

    /// Creates a client with explicit configuration.
    #[must_use]
    pub fn with_config(backend: Arc<dyn Backend>, config: ClientConfig) -> Self {
        Self {
            backend,
            config,
            sessions: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
        }
    }

    /// Scans for devices.
    pub async fn scan(
        &self,
        filter: ScanFilter,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, RuntimeError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(RuntimeError::Closed);
        }
        self.backend.scan(filter, timeout).await.map_err(Into::into)
    }

    /// Connects to a Coyote V3 device and initializes a session.
    pub async fn connect_coyote_v3(
        &self,
        device: &DiscoveredDevice,
        config: CoyoteV3Config,
    ) -> Result<CoyoteV3Session, RuntimeError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(RuntimeError::Closed);
        }
        let link = self.backend.connect(device).await?;
        let info = DeviceInfo::new(
            DeviceId::ble(device.id.as_str()),
            DeviceKind::CoyoteV3,
            device
                .name
                .clone()
                .unwrap_or_else(|| "Coyote V3".to_owned()),
        );
        let target = ReconnectTarget {
            device_id: device.id.clone(),
            filter: ScanFilter {
                name_prefixes: vec![DEVICE_NAME.to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
        };
        let session = CoyoteV3Session::start(SessionStart {
            backend: Arc::clone(&self.backend),
            link,
            info,
            config,
            reconnect_target: target,
            init_timeout: self.config.connect_timeout,
            command_capacity: self.config.command_capacity,
            event_capacity: self.config.event_capacity,
        })
        .await?;
        self.sessions
            .lock()
            .expect("session list mutex poisoned")
            .push(session.clone());
        Ok(session)
    }

    /// Shuts down all sessions created by this client.
    pub async fn shutdown(self) {
        self.closed.store(true, Ordering::SeqCst);
        let sessions = {
            let mut sessions = self.sessions.lock().expect("session list mutex poisoned");
            std::mem::take(&mut *sessions)
        };
        for session in sessions {
            let _ = session.shutdown().await;
        }
    }
}
