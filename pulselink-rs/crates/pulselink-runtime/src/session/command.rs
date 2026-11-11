//! Session command channel.

use pulselink_core::{Channel, Intensity, IntensityDelta};
use pulselink_pattern::Pattern;
use tokio::sync::oneshot;

use crate::config::CoyoteV3Balance;
use crate::error::RuntimeError;
use crate::state::{IntensityChange, PatternId, PlayOptions, StopOutcome};

pub(super) enum SessionCommand {
    SetIntensity {
        channel: Channel,
        intensity: Intensity,
        reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
    },
    AdjustIntensity {
        channel: Channel,
        delta: IntensityDelta,
        reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
    },
    SetSoftLimit {
        channel: Channel,
        limit: Intensity,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    },
    SetBalance {
        balance: CoyoteV3Balance,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    },
    Play {
        pattern: Pattern,
        options: PlayOptions,
        reply: oneshot::Sender<Result<PatternId, RuntimeError>>,
    },
    Stop {
        reply: oneshot::Sender<Result<StopOutcome, RuntimeError>>,
    },
    StopPlayback {
        id: PatternId,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    },
    Reconnect {
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    },
    Shutdown {
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    },
}
