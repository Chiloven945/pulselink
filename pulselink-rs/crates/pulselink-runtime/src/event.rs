//! Session events.

use std::time::Duration;

use pulselink_core::DeviceEvent;
use pulselink_protocol::coyote::v3::Sequence;

use crate::state::{PatternId, PlayEndReason};

/// An event emitted by a Coyote V3 session.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoyoteV3Event {
    /// A common device event.
    Device(DeviceEvent),
    /// A reconnect attempt is scheduled.
    Reconnecting {
        /// One-based attempt number.
        attempt: u32,
        /// Delay before the attempt.
        delay: Duration,
    },
    /// The link was restored and initialization completed.
    Reconnected,
    /// Automatic reconnection gave up.
    ReconnectFailed {
        /// Number of attempts made.
        attempts: u32,
        /// Human-readable error.
        error: String,
    },
    /// A pattern started.
    PatternStarted {
        /// Pattern id.
        id: PatternId,
    },
    /// A pattern ended.
    PatternFinished {
        /// Pattern id.
        id: PatternId,
        /// Why it ended.
        reason: PlayEndReason,
    },
    /// The scheduler fell behind and paused playback.
    SchedulerLag {
        /// How far behind the scheduler was.
        behind: Duration,
    },
    /// An intensity ACK timed out.
    AckTimeout {
        /// Sequence that timed out.
        sequence: Sequence,
    },
    /// A B1 sequence did not match the pending request.
    B1Mismatch {
        /// Expected sequence.
        expected: Sequence,
        /// Received sequence.
        received: Sequence,
    },
    /// The session closed.
    Closed,
}
