//! Notification pump and dispatch.

use futures_util::StreamExt;
use pulselink_core::{BatteryState, DeviceEvent, Observed};
use pulselink_protocol::coyote::v3::decode_battery;
use pulselink_transport::NotificationStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::event::CoyoteV3Event;

use super::actor::Actor;

pub(super) enum Notification {
    B1(Vec<u8>),
    Battery(Vec<u8>),
    Ended,
}

impl Actor {
    pub(super) async fn handle_notification(&mut self, notification: Notification) {
        match notification {
            Notification::B1(bytes) => self.handle_b1(&bytes),
            Notification::Battery(bytes) => {
                if let Ok(level) = decode_battery(&bytes) {
                    self.state.common.battery = Observed::Known(BatteryState::new(level));
                    self.publish();
                    self.emit(CoyoteV3Event::Device(DeviceEvent::BatteryChanged {
                        battery: BatteryState::new(level),
                    }));
                }
            }
            Notification::Ended => self.link_lost("notification stream ended".to_owned()).await,
        }
    }
}

pub(super) fn spawn_notification_pump(
    mut notify: NotificationStream,
    mut battery: Option<NotificationStream>,
    sender: mpsc::Sender<Notification>,
    cancel: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                item = notify.next() => {
                    match item {
                        Some(Ok(bytes)) => { if sender.send(Notification::B1(bytes)).await.is_err() { return; } }
                        Some(Err(_error)) => {
                            let _ = sender.send(Notification::Ended).await;
                            return;
                        }
                        None => { let _ = sender.send(Notification::Ended).await; return; }
                    }
                }
                item = async {
                    match battery.as_mut() {
                        Some(stream) => stream.next().await,
                        None => std::future::pending().await,
                    }
                } => {
                    match item {
                        Some(Ok(bytes)) => {
                            if sender.send(Notification::Battery(bytes)).await.is_err() {
                                return;
                            }
                        }
                        Some(Err(_)) => {}
                        None => battery = None,
                    }
                }
            }
        }
    })
}
