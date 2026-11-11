//! 100 ms scheduler and B0 output.

use pulselink_core::Channel;
use pulselink_pattern::{FRAME_DURATION, StereoFrame};
use pulselink_protocol::coyote::v3::gatt::CHAR_WRITE;
use pulselink_protocol::coyote::v3::{
    B0Command, ChannelWaveform, Sequence, StereoIntensityOp, StereoWaveform,
};
use pulselink_transport::WriteMode;
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use crate::error::RuntimeError;
use crate::scheduler::{frame_index, is_lagging};
use crate::state::PlayEndReason;

use super::actor::Actor;
use super::b0::{keep_intensity, waveform};

impl Actor {
    pub(super) fn ensure_ticker(&mut self) {
        if !self.ticker_active {
            self.ticker = interval_at(
                Instant::now() + self.config.tick_interval,
                self.config.tick_interval,
            );
            self.ticker
                .set_missed_tick_behavior(MissedTickBehavior::Skip);
            self.ticker_active = true;
            self.last_tick = Instant::now();
        }
    }

    pub(super) async fn handle_tick(&mut self) {
        if let Some(pending) = &self.pending
            && Instant::now() >= pending.deadline
        {
            self.timeout_pending();
            return;
        }

        let now = Instant::now();
        if is_lagging(
            now.saturating_duration_since(self.last_tick),
            self.config.lag_threshold,
        ) {
            let behind = now.duration_since(self.last_tick);
            self.pause_for_lag(behind);
            let _ = self.send_silence().await;
            self.last_tick = now;
            self.refresh_ticker();
            return;
        }

        let mut frame = StereoFrame::silent();
        let mut finished = None;
        if let Some(playback) = &mut self.playground {
            let elapsed = now.duration_since(playback.started_at);
            match playback.pattern.frame_at(elapsed).copied() {
                Some(current) => {
                    frame = current;
                    let index =
                        frame_index(elapsed, FRAME_DURATION).unwrap_or(playback.last_frame_index);
                    playback.last_frame_index = index;
                    if let Some(pattern) = &mut self.state.pattern {
                        pattern.elapsed = elapsed;
                        pattern.frame_index = playback.last_frame_index;
                    }
                }
                None => finished = Some(PlayEndReason::Completed),
            }
        }
        if let Some(reason) = finished {
            self.finish_playback(reason);
            frame = StereoFrame::silent();
        }

        let (sequence, intensity) = self.next_intensity_op(now);
        if let Err(error) = self.write_b0(sequence, intensity, frame).await {
            self.link_lost(error.to_string()).await;
            return;
        }
        self.last_tick = now;
        self.publish();
        self.refresh_ticker();
    }

    pub(super) async fn write_b0(
        &mut self,
        sequence: Sequence,
        intensity: StereoIntensityOp,
        frame: StereoFrame,
    ) -> Result<(), RuntimeError> {
        let Some(link) = &self.link else {
            return Err(RuntimeError::NotReady);
        };
        let command = B0Command {
            sequence,
            intensity,
            waveform: StereoWaveform {
                a: waveform(frame.a),
                b: waveform(frame.b),
            },
        };
        link.write(CHAR_WRITE, &command.encode(), WriteMode::Auto)
            .await
            .map_err(RuntimeError::from)
    }

    pub(super) async fn send_silence(&mut self) -> Result<(), RuntimeError> {
        let Some(link) = &self.link else {
            return Ok(());
        };
        let command = B0Command {
            sequence: Sequence::ZERO,
            intensity: keep_intensity(),
            waveform: StereoWaveform {
                a: ChannelWaveform::Silent,
                b: ChannelWaveform::Silent,
            },
        };
        link.write(CHAR_WRITE, &command.encode(), WriteMode::Auto)
            .await
            .map_err(RuntimeError::from)
    }

    pub(super) fn refresh_ticker(&mut self) {
        let has_output = self.playground.is_some()
            || self.pending.is_some()
            || !self.queue.is_empty()
            || Channel::ALL.into_iter().any(|channel| {
                self.state.common.channels[channel]
                    .intensity
                    .value()
                    .is_some_and(|value| !value.is_zero())
            });
        self.ticker_active = has_output;
        if self.ticker_active {
            self.ensure_ticker();
        }
    }
}
