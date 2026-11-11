//! Runtime and session configuration.

use std::time::Duration;

use pulselink_core::{ChannelMap, Intensity, IntensityDelta};

/// Client-level configuration.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Capacity of each session command queue.
    pub command_capacity: usize,
    /// Capacity of each session event broadcast channel.
    pub event_capacity: usize,
    /// Timeout for session initialization.
    pub connect_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            command_capacity: 64,
            event_capacity: 256,
            connect_timeout: Duration::from_secs(15),
        }
    }
}

/// Coyote V3 session configuration.
#[derive(Clone, Debug)]
pub struct CoyoteV3Config {
    /// How long an intensity request waits for a matching B1.
    pub ack_timeout: Duration,
    /// Scheduler tick interval.
    pub tick_interval: Duration,
    /// Maximum scheduler lateness before playback is paused.
    pub lag_threshold: Duration,
    /// Safety policy applied before every intensity operation.
    pub safety: SafetyPolicy,
    /// Automatic reconnection policy.
    pub reconnect: ReconnectPolicy,
    /// Initial per-channel device soft limits written in BF.
    pub initial_soft_limit: ChannelMap<Intensity>,
    /// Optional frequency/intensity balance written in BF.
    pub balance: Option<CoyoteV3Balance>,
}

impl Default for CoyoteV3Config {
    fn default() -> Self {
        Self {
            ack_timeout: Duration::from_secs(1),
            tick_interval: Duration::from_millis(100),
            lag_threshold: Duration::from_millis(500),
            safety: SafetyPolicy::default(),
            reconnect: ReconnectPolicy::default(),
            initial_soft_limit: ChannelMap::new(Intensity::MAX, Intensity::MAX),
            balance: None,
        }
    }
}

/// BF frequency and intensity balance parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoyoteV3Balance {
    /// Frequency balance per channel.
    pub frequency: ChannelMap<u8>,
    /// Intensity balance per channel.
    pub intensity: ChannelMap<u8>,
}

impl Default for CoyoteV3Balance {
    fn default() -> Self {
        Self {
            frequency: ChannelMap::new(128, 128),
            intensity: ChannelMap::new(128, 128),
        }
    }
}

/// Safety limits independent of device-side BF soft limits.
#[derive(Clone, Debug)]
pub struct SafetyPolicy {
    /// Maximum allowed absolute intensity per channel.
    pub max_intensity: ChannelMap<Intensity>,
    /// Optional maximum absolute relative change per channel.
    pub max_delta: Option<ChannelMap<IntensityDelta>>,
    /// Reserved for temporary intensity operations.
    pub allow_temporary: bool,
    /// Whether a successful BF write is required before `Ready`.
    pub require_soft_limit: bool,
}

impl Default for SafetyPolicy {
    fn default() -> Self {
        Self {
            max_intensity: ChannelMap::new(Intensity::MAX, Intensity::MAX),
            max_delta: None,
            allow_temporary: true,
            require_soft_limit: false,
        }
    }
}

/// Automatic reconnection policy.
#[derive(Clone, Debug)]
pub struct ReconnectPolicy {
    /// Whether automatic reconnection is enabled.
    pub enabled: bool,
    /// Maximum number of attempts.
    pub max_attempts: u32,
    /// Initial backoff.
    pub initial_backoff: Duration,
    /// Maximum backoff.
    pub max_backoff: Duration,
    /// Whether to add deterministic jitter to backoff delays.
    pub jitter: bool,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            jitter: true,
        }
    }
}
