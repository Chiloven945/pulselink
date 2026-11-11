//! Session command handling.

use std::time::Duration;

use pulselink_core::{Channel, ConnectionState, DeviceEvent, Intensity, IntensityDelta, Observed};
use pulselink_pattern::Pattern;
use pulselink_protocol::coyote::v3::IntensityOp;
use tokio::sync::oneshot;
use tokio::time::Instant;

use crate::config::CoyoteV3Balance;
use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::safety::{check_absolute, check_delta};
use crate::state::{
    IntensityChange, PatternId, PatternPlaybackState, PlayEndReason, PlayOptions, PlaybackState,
    StopOutcome,
};

use super::ack::IntensityRequest;
use super::actor::Actor;
use super::command::SessionCommand;
use super::playback::Playback;

impl Actor {
    pub(super) async fn handle_command(&mut self, command: SessionCommand) -> bool {
        match command {
            SessionCommand::SetIntensity {
                channel,
                intensity,
                reply,
            } => self.handle_set_intensity(channel, intensity, reply),
            SessionCommand::AdjustIntensity {
                channel,
                delta,
                reply,
            } => self.handle_adjust_intensity(channel, delta, reply),
            SessionCommand::SetSoftLimit {
                channel,
                limit,
                reply,
            } => self.handle_set_soft_limit(channel, limit, reply).await,
            SessionCommand::SetBalance { balance, reply } => {
                self.handle_set_balance(balance, reply).await;
            }
            SessionCommand::Play {
                pattern,
                options,
                reply,
            } => self.handle_play(pattern, options, reply),
            SessionCommand::StopPlayback { id, reply } => {
                self.handle_stop_playback(id, reply).await;
            }
            SessionCommand::Stop { reply } => self.handle_stop(reply).await,
            SessionCommand::Reconnect { reply } => self.handle_reconnect(reply),
            SessionCommand::Shutdown { reply } => {
                self.shutdown_actor().await;
                let _ = reply.send(Ok(()));
                return true;
            }
        }
        false
    }

    fn handle_set_intensity(
        &mut self,
        channel: Channel,
        intensity: Intensity,
        reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
    ) {
        if self.state.common.connection != ConnectionState::Ready {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        }
        let soft = self
            .settings
            .map(|settings| *settings.soft_limit.get(channel));
        if let Err(error) = check_absolute(&self.config.safety, channel, intensity, soft) {
            let _ = reply.send(Err(error));
            return;
        }
        self.desired[channel] = Observed::Known(intensity);
        self.enqueue(IntensityRequest {
            channel,
            op: IntensityOp::Set(intensity),
            target: intensity,
            reply,
        });
    }

    fn handle_adjust_intensity(
        &mut self,
        channel: Channel,
        delta: IntensityDelta,
        reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
    ) {
        if self.state.common.connection != ConnectionState::Ready {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        }
        let Some(current) = self.desired[channel].copied() else {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        };
        if let Err(error) = check_delta(&self.config.safety, channel, delta) {
            let _ = reply.send(Err(error));
            return;
        }
        let magnitude = u8::try_from(delta.magnitude()).unwrap_or(u8::MAX);
        let target = if delta.get() >= 0 {
            current.saturating_add(magnitude)
        } else {
            current.saturating_sub(magnitude)
        };
        let soft = self
            .settings
            .map(|settings| *settings.soft_limit.get(channel));
        if let Err(error) = check_absolute(&self.config.safety, channel, target, soft) {
            let _ = reply.send(Err(error));
            return;
        }
        let op = if delta.is_zero() {
            IntensityOp::Keep
        } else if delta.get() > 0 {
            IntensityOp::Increase(Intensity::new(magnitude).unwrap_or(Intensity::MAX))
        } else {
            IntensityOp::Decrease(Intensity::new(magnitude).unwrap_or(Intensity::MAX))
        };
        self.desired[channel] = Observed::Known(target);
        self.enqueue(IntensityRequest {
            channel,
            op,
            target,
            reply,
        });
    }

    async fn handle_set_soft_limit(
        &mut self,
        channel: Channel,
        limit: Intensity,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    ) {
        if self.state.common.connection != ConnectionState::Ready {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        }
        let mut settings = self.settings.unwrap_or_else(|| self.default_settings());
        settings.soft_limit[channel] = limit;
        match self.write_settings(settings).await {
            Ok(()) => {
                self.state.settings = Observed::Known(settings);
                self.state.common.channels[channel].soft_limit = Observed::Known(limit);
                self.publish();
                self.emit(CoyoteV3Event::Device(DeviceEvent::SettingsChanged));
                let _ = reply.send(Ok(()));
            }
            Err(error) => {
                let _ = reply.send(Err(error));
            }
        }
    }

    async fn handle_set_balance(
        &mut self,
        balance: CoyoteV3Balance,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    ) {
        if self.state.common.connection != ConnectionState::Ready {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        }
        let mut settings = self.settings.unwrap_or_else(|| self.default_settings());
        settings.frequency_balance = balance.frequency;
        settings.intensity_balance = balance.intensity;
        match self.write_settings(settings).await {
            Ok(()) => {
                self.state.settings = Observed::Known(settings);
                self.publish();
                self.emit(CoyoteV3Event::Device(DeviceEvent::SettingsChanged));
                let _ = reply.send(Ok(()));
            }
            Err(error) => {
                let _ = reply.send(Err(error));
            }
        }
    }

    fn handle_play(
        &mut self,
        pattern: Pattern,
        options: PlayOptions,
        reply: oneshot::Sender<Result<PatternId, RuntimeError>>,
    ) {
        if self.state.common.connection != ConnectionState::Ready {
            let _ = reply.send(Err(RuntimeError::NotReady));
            return;
        }
        if self.playground.is_some() {
            if !options.replace {
                let _ = reply.send(Err(RuntimeError::Busy));
                return;
            }
            self.finish_playback(PlayEndReason::Replaced);
        }
        let id = PatternId::new(self.next_pattern_id);
        self.next_pattern_id = self.next_pattern_id.saturating_add(1);
        self.playground = Some(Playback {
            id,
            pattern,
            started_at: Instant::now(),
            last_frame_index: 0,
        });
        self.state.pattern = Some(PatternPlaybackState {
            id,
            elapsed: Duration::ZERO,
            frame_index: 0,
            looped: self
                .playground
                .as_ref()
                .is_some_and(|playback| playback.pattern.looped()),
        });
        self.ensure_ticker();
        let _ = self
            .playback_tx
            .send(PlaybackState::Playing { id, frame_index: 0 });
        self.publish();
        self.emit(CoyoteV3Event::PatternStarted { id });
        let _ = reply.send(Ok(id));
    }

    async fn handle_stop_playback(
        &mut self,
        id: PatternId,
        reply: oneshot::Sender<Result<(), RuntimeError>>,
    ) {
        if self
            .playground
            .as_ref()
            .is_some_and(|playback| playback.id == id)
        {
            self.finish_playback(PlayEndReason::Stopped);
            let _ = self.send_silence().await;
        }
        let _ = reply.send(Ok(()));
    }

    async fn handle_stop(&mut self, reply: oneshot::Sender<Result<StopOutcome, RuntimeError>>) {
        let result = self.stop_flow().await;
        let _ = reply.send(result);
    }

    fn handle_reconnect(&mut self, reply: oneshot::Sender<Result<(), RuntimeError>>) {
        if self.state.common.connection == ConnectionState::Ready {
            let _ = reply.send(Ok(()));
        } else if self.reconnect_task.is_some() {
            let _ = reply.send(Err(RuntimeError::Busy));
        } else {
            self.reconnect_reply = Some(reply);
            self.start_reconnect();
        }
    }
}
