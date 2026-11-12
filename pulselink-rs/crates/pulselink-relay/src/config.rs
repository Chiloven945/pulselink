//! Relay configuration.

use std::net::SocketAddr;
use std::time::Duration;

/// Configuration for a [`crate::LocalRelay`].
#[derive(Clone, Debug)]
pub struct RelayConfig {
    /// Address to bind. Use `0.0.0.0:8787` to accept LAN connections.
    pub bind: SocketAddr,
    /// HTTP path for the WebSocket endpoint.
    pub path: String,
    /// Maximum number of apps attached to one controller.
    pub max_apps_per_controller: usize,
    /// Interval between server heartbeat frames.
    pub heartbeat_interval: Duration,
    /// Optional idle timeout for a controller with no app.
    pub idle_timeout: Option<Duration>,
    /// Optional public base URL used in pairing links and QR codes.
    pub public_base_url: Option<String>,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:8787"
                .parse()
                .expect("static socket address is valid"),
            path: "/v4".to_owned(),
            max_apps_per_controller: 8,
            heartbeat_interval: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(300)),
            public_base_url: None,
        }
    }
}
