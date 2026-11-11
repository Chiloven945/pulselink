//! Socket connection options.

use std::time::Duration;

/// Options for [`crate::Socket::connect_with`].
#[derive(Clone, Debug)]
pub struct SocketOptions {
    /// WebSocket URL, including the `ws://` or `wss://` scheme.
    pub url: String,
    /// Timeout for the TCP/TLS/WebSocket handshake and the initial `hello`.
    pub connect_timeout: Duration,
    /// Default timeout for ordinary RPC requests.
    pub response_timeout: Duration,
    /// Optional interval for server-level `ping` frames.
    pub ping_interval: Option<Duration>,
    /// Maximum accepted WebSocket message size in bytes.
    pub max_message_size: usize,
}

impl SocketOptions {
    /// Creates options for a URL with default timeouts.
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Self::default()
        }
    }
}

impl Default for SocketOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            connect_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(8),
            ping_interval: None,
            max_message_size: 8 * 1024 * 1024,
        }
    }
}
