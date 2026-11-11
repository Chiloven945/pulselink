//! Shared notification multiplexing for one BLE link.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use btleplug::api::{Characteristic, Peripheral as BtPeripheral};
use btleplug::platform::Peripheral;
use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::StreamExt;
use pulselink_transport::{NotificationStream, TransportError};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

struct Subscriber {
    characteristic: Uuid,
    sender: UnboundedSender<Result<Vec<u8>, TransportError>>,
}

#[derive(Default)]
struct HubInner {
    subscribers: Vec<Subscriber>,
    pump: Option<JoinHandle<()>>,
}

/// Fans one btleplug notification stream out to per-characteristic subscribers.
#[derive(Default)]
pub(crate) struct NotificationHub {
    inner: Mutex<HubInner>,
}

impl NotificationHub {
    /// Subscribes to a characteristic and returns an independent stream.
    pub(crate) async fn subscribe(
        self: &Arc<Self>,
        peripheral: &Peripheral,
        characteristic: &Characteristic,
        operation_timeout: Duration,
    ) -> Result<NotificationStream, TransportError> {
        timeout(operation_timeout, peripheral.subscribe(characteristic))
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|error| TransportError::Notify(error.to_string()))?;

        let (sender, receiver) = unbounded();
        let mut inner = self.inner.lock().expect("notification hub mutex poisoned");
        inner.subscribers.push(Subscriber {
            characteristic: characteristic.uuid,
            sender,
        });

        let needs_pump = inner.pump.as_ref().is_none_or(JoinHandle::is_finished);
        if needs_pump {
            if let Some(old) = inner.pump.take() {
                old.abort();
            }
            let hub = Arc::clone(self);
            let peripheral = peripheral.clone();
            inner.pump = Some(tokio::spawn(async move {
                hub.run_pump(peripheral).await;
            }));
        }
        drop(inner);

        Ok(Box::pin(receiver))
    }

    /// Stops the pump and terminates every active notification stream.
    pub(crate) fn shutdown(&self) {
        let mut inner = self.inner.lock().expect("notification hub mutex poisoned");
        if let Some(pump) = inner.pump.take() {
            pump.abort();
        }
        fail_subscribers(&mut inner.subscribers, &TransportError::Disconnected);
    }

    async fn run_pump(self: Arc<Self>, peripheral: Peripheral) {
        let mut notifications = match peripheral.notifications().await {
            Ok(notifications) => notifications,
            Err(error) => {
                self.fail_all(&TransportError::Notify(error.to_string()));
                return;
            }
        };

        while let Some(notification) = notifications.next().await {
            self.dispatch(notification.uuid, &notification.value);
        }

        self.fail_all(&TransportError::Disconnected);
    }

    fn dispatch(&self, characteristic: Uuid, value: &[u8]) {
        let mut inner = self.inner.lock().expect("notification hub mutex poisoned");
        inner.subscribers.retain_mut(|subscriber| {
            if subscriber.characteristic == characteristic {
                subscriber.sender.unbounded_send(Ok(value.to_vec())).is_ok()
            } else {
                !subscriber.sender.is_closed()
            }
        });
    }

    fn fail_all(&self, error: &TransportError) {
        let mut inner = self.inner.lock().expect("notification hub mutex poisoned");
        inner.pump = None;
        fail_subscribers(&mut inner.subscribers, error);
    }

    #[cfg(test)]
    pub(crate) fn subscribe_for_test(&self, characteristic: Uuid) -> NotificationStream {
        let (sender, receiver) = unbounded();
        self.inner
            .lock()
            .expect("notification hub mutex poisoned")
            .subscribers
            .push(Subscriber {
                characteristic,
                sender,
            });
        Box::pin(receiver)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_for_test(&self, characteristic: Uuid, value: &[u8]) {
        self.dispatch(characteristic, value);
    }

    #[cfg(test)]
    pub(crate) fn fail_all_for_test(&self, error: &TransportError) {
        self.fail_all(error);
    }
}

fn fail_subscribers(subscribers: &mut Vec<Subscriber>, error: &TransportError) {
    for subscriber in subscribers.drain(..) {
        let _ = subscriber
            .sender
            .unbounded_send(Err(clone_transport_error(error)));
    }
}

fn clone_transport_error(error: &TransportError) -> TransportError {
    match error {
        TransportError::NoAdapter => TransportError::NoAdapter,
        TransportError::Scan(message) => TransportError::Scan(message.clone()),
        TransportError::DeviceNotFound(id) => TransportError::DeviceNotFound(id.clone()),
        TransportError::Connect(message) => TransportError::Connect(message.clone()),
        TransportError::Gatt(message) => TransportError::Gatt(message.clone()),
        TransportError::Notify(message) => TransportError::Notify(message.clone()),
        TransportError::Disconnected => TransportError::Disconnected,
        TransportError::Timeout => TransportError::Timeout,
        TransportError::Backend(error) => TransportError::Backend(error.to_string().into()),
        _ => TransportError::Backend("unknown transport error".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    const CHAR: Uuid = Uuid::from_u128(0x150b);

    #[tokio::test]
    async fn fans_out_to_matching_subscribers_only() {
        let hub = Arc::new(NotificationHub::default());
        let mut first = hub.subscribe_for_test(CHAR);
        let mut second = hub.subscribe_for_test(CHAR);
        let mut other = hub.subscribe_for_test(Uuid::from_u128(0x1500));

        hub.dispatch_for_test(CHAR, &[1, 2, 3]);

        assert_eq!(
            first.next().await.expect("first item").expect("ok"),
            vec![1, 2, 3]
        );
        assert_eq!(
            second.next().await.expect("second item").expect("ok"),
            vec![1, 2, 3]
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(10), other.next())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn shutdown_terminates_streams() {
        let hub = Arc::new(NotificationHub::default());
        let mut stream = hub.subscribe_for_test(CHAR);
        hub.fail_all_for_test(&TransportError::Disconnected);
        assert!(matches!(
            stream.next().await.expect("item"),
            Err(TransportError::Disconnected)
        ));
    }
}
