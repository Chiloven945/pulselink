//! Link loss and reconnection.

use std::sync::Arc;

use pulselink_core::{Channel, ChannelMap, ConnectionState, DeviceEvent, Observed};
use pulselink_transport::{GattLink, TransportError};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::warn;

use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::state::PlayEndReason;

use super::actor::Actor;

pub(super) enum ReconnectResult {
    Success(Box<dyn GattLink>),
    Failed { attempts: u32, error: String },
}

impl Actor {
    pub(super) async fn link_lost(&mut self, message: String) {
        if matches!(
            self.state.common.connection,
            ConnectionState::Disconnected | ConnectionState::Reconnecting
        ) {
            return;
        }
        warn!(%message, "Coyote V3 link lost");
        self.ticker_active = false;
        self.next_deadline = None;
        self.cancel_intensity(RuntimeError::Transport(TransportError::Disconnected));
        self.finish_playback(PlayEndReason::LinkLost);
        for channel in Channel::ALL {
            self.state.common.channels[channel].intensity = Observed::Unknown;
        }
        self.desired = ChannelMap::new(Observed::Unknown, Observed::Unknown);
        self.state.common.connection = ConnectionState::Degraded;
        self.state.pattern = None;
        self.publish();
        self.emit(CoyoteV3Event::Device(DeviceEvent::ConnectionChanged {
            state: ConnectionState::Degraded,
        }));
        if let Some(link) = self.link.take() {
            let _ = timeout(self.config.ack_timeout, link.disconnect()).await;
        }
        self.stop_notification_task();
        if self.config.reconnect.enabled {
            self.start_reconnect();
        } else {
            self.state.common.connection = ConnectionState::Disconnected;
            self.publish();
            self.emit(CoyoteV3Event::Device(DeviceEvent::ConnectionChanged {
                state: ConnectionState::Disconnected,
            }));
        }
    }

    pub(super) fn start_reconnect(&mut self) {
        if self.reconnect_task.is_some() {
            return;
        }
        self.state.common.connection = ConnectionState::Reconnecting;
        self.publish();
        let (tx, rx) = mpsc::channel(8);
        self.reconnect_tx = tx;
        self.reconnect_rx = rx;
        let backend = Arc::clone(&self.backend);
        let target = self.reconnect_target.clone();
        let policy = self.config.reconnect.clone();
        let events = self.events.clone();
        let cancel = self.cancel.clone();
        let sender = self.reconnect_tx.clone();
        self.reconnect_task = Some(tokio::spawn(async move {
            let mut last_error = String::new();
            for attempt in 1..=policy.max_attempts.max(1) {
                let base = policy
                    .initial_backoff
                    .saturating_mul(1_u32 << attempt.saturating_sub(1).min(16));
                let delay = base.min(policy.max_backoff);
                let delay = if policy.jitter && attempt % 2 == 0 {
                    delay + delay / 10
                } else {
                    delay
                };
                let delay = delay.min(policy.max_backoff);
                let _ = events.send(CoyoteV3Event::Reconnecting { attempt, delay });
                tokio::select! {
                    () = cancel.cancelled() => return,
                    () = tokio::time::sleep(delay) => {}
                }
                match backend.reconnect(&target).await {
                    Ok(link) => {
                        let _ = sender.send(ReconnectResult::Success(link)).await;
                        return;
                    }
                    Err(error) => {
                        last_error = error.to_string();
                    }
                }
            }
            let _ = sender
                .send(ReconnectResult::Failed {
                    attempts: policy.max_attempts.max(1),
                    error: last_error,
                })
                .await;
        }));
    }

    pub(super) async fn handle_reconnect_result(&mut self, result: ReconnectResult) {
        if let Some(task) = self.reconnect_task.take() {
            task.abort();
        }
        match result {
            ReconnectResult::Success(link) => match self.initialize(link).await {
                Ok(()) => {
                    self.emit(CoyoteV3Event::Reconnected);
                    if let Some(reply) = self.reconnect_reply.take() {
                        let _ = reply.send(Ok(()));
                    }
                }
                Err(error) => {
                    self.state.common.connection = ConnectionState::Disconnected;
                    self.publish();
                    self.emit(CoyoteV3Event::ReconnectFailed {
                        attempts: self.config.reconnect.max_attempts,
                        error: error.to_string(),
                    });
                    if let Some(reply) = self.reconnect_reply.take() {
                        let _ = reply.send(Err(RuntimeError::ReconnectFailed(
                            self.config.reconnect.max_attempts,
                        )));
                    }
                }
            },
            ReconnectResult::Failed { attempts, error } => {
                self.state.common.connection = ConnectionState::Disconnected;
                self.publish();
                self.emit(CoyoteV3Event::ReconnectFailed {
                    attempts,
                    error: error.clone(),
                });
                if let Some(reply) = self.reconnect_reply.take() {
                    let _ = reply.send(Err(RuntimeError::ReconnectFailed(attempts)));
                }
            }
        }
    }
}
