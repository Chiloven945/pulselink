//! Local DG-LAB V4 WebSocket relay.
//!
//! The relay accepts controller connections on `/v4`, pairs real DG-LAB app
//! connections by `tid`, and forwards V4 messages between them. It is intended
//! for LAN testing and is not a replacement for the official relay.

mod config;
mod error;
mod registry;
mod server;
mod url;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;

pub use config::RelayConfig;
pub use error::RelayError;

/// A lifecycle event emitted by a [`LocalRelay`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RelayEvent {
    /// A controller connected and received a target id.
    ControllerConnected {
        /// Pairing target id.
        target_id: String,
    },
    /// A controller disconnected.
    ControllerDisconnected {
        /// Pairing target id.
        target_id: String,
    },
    /// An app attached to a controller.
    AppAttached {
        /// Pairing target id.
        target_id: String,
        /// App client id.
        client_id: String,
    },
    /// An app detached from a controller.
    AppDisconnected {
        /// Pairing target id.
        target_id: String,
        /// App client id.
        client_id: String,
    },
}

/// A running local relay.
pub struct LocalRelay {
    addr: SocketAddr,
    config: RelayConfig,
    shutdown: watch::Sender<bool>,
    state: Arc<registry::RelayState>,
    task: Option<JoinHandle<()>>,
    events: broadcast::Sender<RelayEvent>,
}

impl LocalRelay {
    /// Binds the configured address and starts accepting connections.
    pub async fn start(config: RelayConfig) -> Result<Self, RelayError> {
        let listener = tokio::net::TcpListener::bind(config.bind)
            .await
            .map_err(RelayError::Io)?;
        let addr = listener.local_addr().map_err(RelayError::Io)?;
        let (events, _) = broadcast::channel(64);
        let (shutdown, shutdown_rx) = watch::channel(false);
        let state = Arc::new(registry::RelayState::new(config.clone(), events.clone()));
        let task = tokio::spawn(server::run(listener, Arc::clone(&state), shutdown_rx));

        Ok(Self {
            addr,
            config,
            shutdown,
            state,
            task: Some(task),
            events,
        })
    }

    /// Returns the bound local address.
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the controller WebSocket URL.
    #[must_use]
    pub fn controller_url(&self) -> String {
        url::controller_url(&self.config, self.addr)
    }

    /// Returns the app pairing URL for a target id.
    #[must_use]
    pub fn pairing_url(&self, target_id: &str) -> String {
        url::pairing_url(&self.controller_url(), target_id)
    }

    /// Returns the official DG-LAB QR redirect URL for a pairing URL.
    #[must_use]
    pub fn qr_url(&self, target_id: &str) -> String {
        url::qr_url(&self.pairing_url(target_id))
    }

    /// Subscribes to relay lifecycle events.
    #[must_use]
    pub fn events(&self) -> broadcast::Receiver<RelayEvent> {
        self.events.subscribe()
    }

    /// Stops accepting connections and closes the relay.
    pub async fn shutdown(mut self) -> Result<(), RelayError> {
        let _ = self.shutdown.send(true);
        self.state.shutdown_all();
        if let Some(task) = self.task.take()
            && let Ok(result) = timeout(Duration::from_secs(2), task).await
        {
            result.map_err(|error| RelayError::Task(error.to_string()))?;
        }
        Ok(())
    }
}

impl Drop for LocalRelay {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
        self.state.shutdown_all();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
