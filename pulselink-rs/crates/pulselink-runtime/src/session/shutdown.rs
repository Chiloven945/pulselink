//! Stop and shutdown flows.

use pulselink_core::{Channel, ChannelMap, ConnectionState, Intensity};
use pulselink_pattern::StereoFrame;
use pulselink_protocol::coyote::v3::{IntensityOp, StereoIntensityOp};
use tokio::time::timeout;

use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::state::{PlayEndReason, StopOutcome};

use super::actor::Actor;

impl Actor {
    pub(super) async fn stop_flow(&mut self) -> Result<StopOutcome, RuntimeError> {
        if self.state.common.connection != ConnectionState::Ready {
            return Err(RuntimeError::NotReady);
        }
        self.finish_playback(PlayEndReason::Stopped);
        self.cancel_intensity(RuntimeError::Cancelled);
        self.ticker_active = false;
        self.next_deadline = None;

        if self.link.is_none() {
            return Err(RuntimeError::NotReady);
        }
        let sequence = self.sequence.next();
        let intensity = StereoIntensityOp {
            a: IntensityOp::Set(Intensity::MIN),
            b: IntensityOp::Set(Intensity::MIN),
        };
        self.write_b0(sequence, intensity, StereoFrame::silent())
            .await?;
        let acknowledged = self.wait_for_b1(sequence).await.is_some();
        self.publish();
        Ok(StopOutcome {
            acknowledged,
            intensity: ChannelMap::new(
                self.state.common.channels[Channel::A].intensity,
                self.state.common.channels[Channel::B].intensity,
            ),
        })
    }

    pub(super) async fn shutdown_actor(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.state.common.connection = ConnectionState::Closing;
        self.publish();
        self.ticker_active = false;
        self.cancel_intensity(RuntimeError::Closed);
        self.finish_playback(PlayEndReason::Shutdown);
        if self.link.is_some() {
            let sequence = self.sequence.next();
            let intensity = StereoIntensityOp {
                a: IntensityOp::Set(Intensity::MIN),
                b: IntensityOp::Set(Intensity::MIN),
            };
            let _ = self
                .write_b0(sequence, intensity, StereoFrame::silent())
                .await;
            let _ = self.wait_for_b1(sequence).await;
        }
        if let Some(link) = self.link.take() {
            let _ = timeout(self.config.ack_timeout, link.disconnect()).await;
        }
        self.stop_notification_task();
        if let Some(task) = self.reconnect_task.take() {
            task.abort();
        }
        self.state.common.connection = ConnectionState::Disconnected;
        self.publish();
        self.emit(CoyoteV3Event::Closed);
    }

    pub(super) fn stop_notification_task(&mut self) {
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
    }
}
