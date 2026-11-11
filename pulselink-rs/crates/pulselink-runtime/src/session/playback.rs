//! Pattern playback state.

use pulselink_pattern::Pattern;
use std::time::Duration;
use tokio::time::Instant;

use crate::event::CoyoteV3Event;
use crate::state::{PatternId, PlayEndReason, PlaybackState};

use super::actor::Actor;

pub(super) struct Playback {
    pub(super) id: PatternId,
    pub(super) pattern: Pattern,
    pub(super) started_at: Instant,
    pub(super) last_frame_index: usize,
}

impl Actor {
    pub(super) fn finish_playback(&mut self, reason: PlayEndReason) {
        let Some(playback) = self.playground.take() else {
            return;
        };
        self.state.pattern = None;
        self.playback_ends
            .lock()
            .expect("playback ends mutex poisoned")
            .insert(playback.id, reason);
        let _ = self.playback_tx.send(PlaybackState::Finished {
            id: playback.id,
            reason,
        });
        self.emit(CoyoteV3Event::PatternFinished {
            id: playback.id,
            reason,
        });
        self.publish();
    }

    pub(super) fn pause_for_lag(&mut self, behind: Duration) {
        self.finish_playback(PlayEndReason::SchedulerLag);
        self.emit(CoyoteV3Event::SchedulerLag { behind });
    }
}
