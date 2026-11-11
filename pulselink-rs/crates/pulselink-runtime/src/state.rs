//! Public session state and playback values.

use std::time::Duration;

use pulselink_core::{Channel, ChannelMap, DeviceState, Intensity, Observed};
use pulselink_protocol::coyote::v3::B1Message;

/// Identifies one pattern playback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PatternId(u64);

impl PatternId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Why a pattern playback ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayEndReason {
    /// The pattern reached its end.
    Completed,
    /// The user stopped playback.
    Stopped,
    /// A newer pattern replaced this one.
    Replaced,
    /// The scheduler fell behind.
    SchedulerLag,
    /// The device link was lost.
    LinkLost,
    /// The session shut down.
    Shutdown,
    /// Playback failed.
    Failed,
}

/// Observable state of a playback handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    /// No playback is active.
    Idle,
    /// A pattern is playing.
    Playing {
        /// Pattern id.
        id: PatternId,
        /// Current frame index.
        frame_index: usize,
    },
    /// Playback ended.
    Finished {
        /// Pattern id.
        id: PatternId,
        /// Why it ended.
        reason: PlayEndReason,
    },
}

impl PlaybackState {
    /// Returns whether the state is terminal.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        matches!(self, Self::Finished { .. })
    }
}

/// Options for starting a pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayOptions {
    /// Replace an active pattern instead of returning `Busy`.
    pub replace: bool,
}

/// The result of a stop request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StopOutcome {
    /// Whether the device acknowledged the zero-intensity B0.
    pub acknowledged: bool,
    /// Last observed intensity after the stop attempt.
    pub intensity: ChannelMap<Observed<Intensity>>,
}

/// A completed intensity change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntensityChange {
    /// Channel that changed.
    pub channel: Channel,
    /// Requested target intensity.
    pub requested: Intensity,
    /// Intensity reported by the device.
    pub applied: Intensity,
}

/// Current BF settings known to the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoyoteV3Settings {
    /// Per-channel device soft limit.
    pub soft_limit: ChannelMap<Intensity>,
    /// Per-channel frequency balance.
    pub frequency_balance: ChannelMap<u8>,
    /// Per-channel intensity balance.
    pub intensity_balance: ChannelMap<u8>,
}

/// Public snapshot of a Coyote V3 session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoyoteV3State {
    /// Common device state.
    pub common: DeviceState,
    /// Last settings written to the device, if initialization succeeded.
    pub settings: Observed<CoyoteV3Settings>,
    /// Active pattern playback, if any.
    pub pattern: Option<PatternPlaybackState>,
    /// Last B1 notification.
    pub last_b1: Option<B1Message>,
}

/// Playback progress included in [`CoyoteV3State`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternPlaybackState {
    /// Pattern id.
    pub id: PatternId,
    /// Elapsed playback time.
    pub elapsed: Duration,
    /// Current frame index.
    pub frame_index: usize,
    /// Whether the pattern loops.
    pub looped: bool,
}
