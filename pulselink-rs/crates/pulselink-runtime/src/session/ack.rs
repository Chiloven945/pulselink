//! Intensity queue and B1 acknowledgement state.

use pulselink_core::{Channel, DeviceEvent, Intensity, IntensitySource, Observed};
use pulselink_protocol::coyote::v3::{B1Message, IntensityOp, Sequence, StereoIntensityOp};
use tokio::sync::oneshot;
use tokio::time::Instant;

use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::state::IntensityChange;

use super::actor::Actor;
use super::b0::{keep_intensity, op_for_channel};
use super::notification::Notification;

const INTENSITY_QUEUE_CAPACITY: usize = 16;

pub(super) struct IntensityRequest {
    pub(super) channel: Channel,
    pub(super) op: IntensityOp,
    pub(super) target: Intensity,
    pub(super) reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
}

pub(super) struct PendingIntensity {
    pub(super) sequence: Sequence,
    pub(super) channel: Channel,
    pub(super) target: Intensity,
    pub(super) reply: oneshot::Sender<Result<IntensityChange, RuntimeError>>,
    pub(super) deadline: Instant,
}

impl Actor {
    pub(super) fn enqueue(&mut self, request: IntensityRequest) {
        if self.queue.len() >= INTENSITY_QUEUE_CAPACITY {
            let _ = request.reply.send(Err(RuntimeError::Busy));
            return;
        }
        self.queue.push_back(request);
        self.ensure_ticker();
    }

    pub(super) fn handle_b1(&mut self, bytes: &[u8]) {
        let message = match B1Message::decode(bytes) {
            Ok(message) => message,
            Err(error) => {
                self.emit(CoyoteV3Event::Device(DeviceEvent::Error {
                    message: error.to_string(),
                }));
                return;
            }
        };
        self.state.last_b1 = Some(message);
        for channel in Channel::ALL {
            let intensity = message.intensity[channel];
            self.state.common.channels[channel].intensity = Observed::Known(intensity);
            if self.pending.as_ref().is_none_or(|pending| {
                pending.sequence != message.sequence || pending.channel != channel
            }) {
                self.desired[channel] = Observed::Known(intensity);
            }
            self.emit(CoyoteV3Event::Device(DeviceEvent::IntensityChanged {
                channel,
                intensity: Observed::Known(intensity),
                source: IntensitySource::Device,
            }));
        }
        if let Some(pending) = &self.pending {
            if message.sequence == pending.sequence {
                self.desired[pending.channel] = Observed::Known(message.intensity[pending.channel]);
                let reply = self.pending.take();
                if let Some(pending) = reply {
                    let _ = pending.reply.send(Ok(IntensityChange {
                        channel: pending.channel,
                        requested: pending.target,
                        applied: message.intensity[pending.channel],
                    }));
                }
                self.next_deadline = None;
            } else if message.sequence != Sequence::ZERO {
                self.emit(CoyoteV3Event::B1Mismatch {
                    expected: pending.sequence,
                    received: message.sequence,
                });
            }
        }
        self.publish();
    }

    pub(super) fn next_intensity_op(&mut self, now: Instant) -> (Sequence, StereoIntensityOp) {
        if self.pending.is_some() {
            return (Sequence::ZERO, keep_intensity());
        }
        let Some(request) = self.queue.pop_front() else {
            return (Sequence::ZERO, keep_intensity());
        };
        let sequence = self.sequence.next();
        let mut op = keep_intensity();
        *op_for_channel(&mut op, request.channel) = request.op;
        self.pending = Some(PendingIntensity {
            sequence,
            channel: request.channel,
            target: request.target,
            reply: request.reply,
            deadline: now + self.config.ack_timeout,
        });
        self.next_deadline = Some(now + self.config.ack_timeout);
        (sequence, op)
    }

    pub(super) fn handle_deadline(&mut self) {
        if let Some(pending) = &self.pending
            && Instant::now() >= pending.deadline
        {
            self.timeout_pending();
        }
    }

    pub(super) fn timeout_pending(&mut self) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        self.next_deadline = None;
        self.state.common.channels[pending.channel].intensity = Observed::Unknown;
        self.desired[pending.channel] = Observed::Unknown;
        self.emit(CoyoteV3Event::AckTimeout {
            sequence: pending.sequence,
        });
        self.emit(CoyoteV3Event::Device(DeviceEvent::IntensityChanged {
            channel: pending.channel,
            intensity: Observed::Unknown,
            source: IntensitySource::Unknown,
        }));
        self.publish();
        let _ = pending
            .reply
            .send(Err(RuntimeError::AckTimeout(pending.sequence)));
    }

    pub(super) async fn wait_for_b1(&mut self, sequence: Sequence) -> Option<B1Message> {
        let deadline = Instant::now() + self.config.ack_timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            tokio::select! {
                () = self.cancel.cancelled() => return None,
                notification = self.notification_rx.recv() => {
                    match notification {
                        Some(Notification::B1(bytes)) => {
                            if let Ok(message) = B1Message::decode(&bytes) {
                                for channel in Channel::ALL {
                                    self.state.common.channels[channel].intensity = Observed::Known(message.intensity[channel]);
                                }
                                if message.sequence == sequence {
                                    self.publish();
                                    return Some(message);
                                }
                            }
                        }
                        Some(Notification::Battery(_)) => {}
                        Some(Notification::Ended) | None => return None,
                    }
                }
                () = tokio::time::sleep(remaining) => return None,
            }
        }
    }

    pub(super) fn cancel_intensity(&mut self, error: RuntimeError) {
        self.next_deadline = None;
        if let Some(pending) = self.pending.take() {
            let _ = pending.reply.send(Err(error));
        }
        while let Some(request) = self.queue.pop_front() {
            let _ = request.reply.send(Err(RuntimeError::Cancelled));
        }
    }
}
