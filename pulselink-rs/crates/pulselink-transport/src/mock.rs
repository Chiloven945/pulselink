//! Deterministic in-memory transport implementations.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::central::{Central, Connector, ReconnectTarget};
use crate::error::TransportError;
use crate::link::{
    DiscoveredDevice, GattLink, NotificationStream, ScanFilter, TransportDeviceId, WriteMode,
};

/// A recorded write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteRecord {
    /// Target characteristic.
    pub characteristic: Uuid,
    /// Written bytes.
    pub data: Vec<u8>,
    /// Requested write mode.
    pub mode: WriteMode,
}

type NotificationSender = UnboundedSender<Result<Vec<u8>, TransportError>>;
type Subscribers = Mutex<HashMap<Uuid, Vec<NotificationSender>>>;

#[derive(Debug)]
struct MockLinkInner {
    device_id: TransportDeviceId,
    connected: AtomicBool,
    mtu: std::sync::atomic::AtomicU16,
    writes: Mutex<Vec<WriteRecord>>,
    fail_next_write: Mutex<Option<TransportError>>,
    read_data: Mutex<HashMap<Uuid, Vec<u8>>>,
    subscribers: Subscribers,
}

/// A controllable in-memory [`GattLink`].
#[derive(Clone, Debug)]
pub struct MockLink {
    inner: Arc<MockLinkInner>,
}

impl MockLink {
    /// Creates a connected mock link.
    #[must_use]
    pub fn new(device_id: TransportDeviceId) -> Self {
        Self {
            inner: Arc::new(MockLinkInner {
                device_id,
                connected: AtomicBool::new(true),
                mtu: std::sync::atomic::AtomicU16::new(185),
                writes: Mutex::new(Vec::new()),
                fail_next_write: Mutex::new(None),
                read_data: Mutex::new(HashMap::new()),
                subscribers: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Injects a notification for a characteristic.
    ///
    /// Subscribers whose receivers have been dropped are pruned. If there are no
    /// subscribers, the notification is discarded.
    pub fn inject_notification(&self, characteristic: Uuid, data: &[u8]) {
        let mut subscribers = self
            .inner
            .subscribers
            .lock()
            .expect("mock subscribers mutex poisoned");

        if let Some(senders) = subscribers.get_mut(&characteristic) {
            senders.retain(|sender| sender.unbounded_send(Ok(data.to_vec())).is_ok());
        }
    }

    /// Returns the writes recorded so far.
    #[must_use]
    pub fn writes(&self) -> Vec<WriteRecord> {
        self.inner
            .writes
            .lock()
            .expect("mock writes mutex poisoned")
            .clone()
    }

    /// Makes the next write fail with the given error.
    pub fn fail_next_write(&self, error: TransportError) {
        *self
            .inner
            .fail_next_write
            .lock()
            .expect("mock failure mutex poisoned") = Some(error);
    }

    /// Sets the reported ATT MTU.
    pub fn set_mtu(&self, mtu: u16) {
        self.inner.mtu.store(mtu, Ordering::SeqCst);
    }

    /// Sets the data returned by [`GattLink::read`] for a characteristic.
    pub fn set_read_data(&self, characteristic: Uuid, data: Vec<u8>) {
        self.inner
            .read_data
            .lock()
            .expect("mock read mutex poisoned")
            .insert(characteristic, data);
    }

    /// Marks the link as connected or disconnected.
    pub fn set_connected(&self, connected: bool) {
        self.inner.connected.store(connected, Ordering::SeqCst);
        if !connected {
            self.inner
                .subscribers
                .lock()
                .expect("mock subscribers mutex poisoned")
                .clear();
        }
    }

    /// Disconnects the link and ends all notification streams.
    pub fn mark_disconnected(&self) {
        self.set_connected(false);
    }
}

#[async_trait]
impl GattLink for MockLink {
    fn device_id(&self) -> &TransportDeviceId {
        &self.inner.device_id
    }

    fn is_connected(&self) -> bool {
        self.inner.connected.load(Ordering::SeqCst)
    }

    fn mtu(&self) -> Option<u16> {
        Some(self.inner.mtu.load(Ordering::SeqCst))
    }

    async fn write(
        &self,
        characteristic: Uuid,
        data: &[u8],
        mode: WriteMode,
    ) -> Result<(), TransportError> {
        {
            let mut writes = self
                .inner
                .writes
                .lock()
                .expect("mock writes mutex poisoned");
            writes.push(WriteRecord {
                characteristic,
                data: data.to_vec(),
                mode,
            });
        }

        if let Some(error) = self
            .inner
            .fail_next_write
            .lock()
            .expect("mock failure mutex poisoned")
            .take()
        {
            return Err(error);
        }

        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        Ok(())
    }

    async fn read(&self, characteristic: Uuid) -> Result<Vec<u8>, TransportError> {
        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        self.inner
            .read_data
            .lock()
            .expect("mock read mutex poisoned")
            .get(&characteristic)
            .cloned()
            .ok_or_else(|| TransportError::Gatt("no mock read data configured".to_owned()))
    }

    async fn subscribe(&self, characteristic: Uuid) -> Result<NotificationStream, TransportError> {
        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        let (sender, receiver) = unbounded();
        self.inner
            .subscribers
            .lock()
            .expect("mock subscribers mutex poisoned")
            .entry(characteristic)
            .or_default()
            .push(sender);

        Ok(notification_stream(receiver))
    }

    async fn disconnect(&self) -> Result<(), TransportError> {
        self.mark_disconnected();
        Ok(())
    }
}

fn notification_stream(
    receiver: UnboundedReceiver<Result<Vec<u8>, TransportError>>,
) -> NotificationStream {
    Box::pin(futures_util::stream::unfold(
        receiver,
        |mut receiver| async move { receiver.next().await.map(|item| (item, receiver)) },
    ))
}

/// A controllable in-memory backend implementing [`Central`] and [`Connector`].
#[derive(Debug, Default)]
pub struct MockBackend {
    devices: Vec<DiscoveredDevice>,
    links: Mutex<VecDeque<MockLink>>,
    last_link: Mutex<Option<MockLink>>,
}

impl MockBackend {
    /// Creates a backend that returns the given devices from scans.
    #[must_use]
    pub fn new(devices: Vec<DiscoveredDevice>) -> Self {
        Self {
            devices,
            links: Mutex::new(VecDeque::new()),
            last_link: Mutex::new(None),
        }
    }

    /// Queues a link to be returned by the next connect/reconnect.
    pub fn push_link(&self, link: MockLink) {
        self.links
            .lock()
            .expect("mock links mutex poisoned")
            .push_back(link);
    }

    /// Returns the most recently returned link.
    #[must_use]
    pub fn last_link(&self) -> Option<MockLink> {
        self.last_link
            .lock()
            .expect("mock last link mutex poisoned")
            .clone()
    }

    fn pop_link(&self) -> Option<MockLink> {
        let link = self
            .links
            .lock()
            .expect("mock links mutex poisoned")
            .pop_front();
        if let Some(link) = &link {
            *self
                .last_link
                .lock()
                .expect("mock last link mutex poisoned") = Some(link.clone());
        }
        link
    }
}

#[async_trait]
impl Central for MockBackend {
    async fn scan(
        &self,
        filter: ScanFilter,
        _timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, TransportError> {
        Ok(self
            .devices
            .iter()
            .filter(|device| filter.matches(device))
            .cloned()
            .collect())
    }

    async fn connect(
        &self,
        device: &DiscoveredDevice,
    ) -> Result<Box<dyn GattLink>, TransportError> {
        self.pop_link()
            .map(|link| Box::new(link) as Box<dyn GattLink>)
            .ok_or_else(|| TransportError::DeviceNotFound(device.id.clone()))
    }
}

#[async_trait]
impl Connector for MockBackend {
    async fn reconnect(
        &self,
        target: &ReconnectTarget,
    ) -> Result<Box<dyn GattLink>, TransportError> {
        self.pop_link()
            .map(|link| Box::new(link) as Box<dyn GattLink>)
            .ok_or_else(|| TransportError::DeviceNotFound(target.device_id.clone()))
    }
}

/// Convenience alias for a mock [`Central`].
pub type MockCentral = MockBackend;
/// Convenience alias for a mock [`Connector`].
pub type MockConnector = MockBackend;

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    fn device(id: &str) -> DiscoveredDevice {
        DiscoveredDevice {
            id: TransportDeviceId::new(id),
            name: Some("47L121000".to_owned()),
            rssi: Some(-40),
            service_uuids: Vec::new(),
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn write_read_subscribe() {
        block_on(async {
            let link = MockLink::new(TransportDeviceId::new("dev"));
            let characteristic = Uuid::from_u128(1);

            link.set_read_data(characteristic, vec![1, 2, 3]);
            assert_eq!(
                link.read(characteristic).await.expect("read"),
                vec![1, 2, 3]
            );

            link.write(characteristic, &[4, 5], WriteMode::Auto)
                .await
                .expect("write");
            assert_eq!(
                link.writes(),
                vec![WriteRecord {
                    characteristic,
                    data: vec![4, 5],
                    mode: WriteMode::Auto,
                }]
            );

            let mut stream = link.subscribe(characteristic).await.expect("subscribe");
            link.inject_notification(characteristic, &[9, 8]);
            assert_eq!(
                stream.next().await.expect("notification").expect("ok"),
                vec![9, 8]
            );
        });
    }

    #[test]
    fn disconnect_ends_streams() {
        block_on(async {
            let link = MockLink::new(TransportDeviceId::new("dev"));
            let characteristic = Uuid::from_u128(1);
            let mut stream = link.subscribe(characteristic).await.expect("subscribe");

            link.disconnect().await.expect("disconnect");
            assert!(!link.is_connected());
            assert!(stream.next().await.is_none());
        });
    }

    #[test]
    fn failure_injection() {
        block_on(async {
            let link = MockLink::new(TransportDeviceId::new("dev"));
            link.fail_next_write(TransportError::Gatt("boom".to_owned()));
            assert!(
                link.write(Uuid::from_u128(1), &[1], WriteMode::Auto)
                    .await
                    .is_err()
            );
            assert!(
                link.write(Uuid::from_u128(1), &[1], WriteMode::Auto)
                    .await
                    .is_ok()
            );
        });
    }

    #[test]
    fn backend_scan_connect_reconnect() {
        block_on(async {
            let backend = MockBackend::new(vec![device("dev-1")]);
            let filter = ScanFilter::default();
            let devices = backend
                .scan(filter.clone(), Duration::from_millis(1))
                .await
                .expect("scan");
            assert_eq!(devices.len(), 1);

            let link = MockLink::new(TransportDeviceId::new("dev-1"));
            backend.push_link(link);
            let _connected = backend.connect(&devices[0]).await.expect("connect");
            assert!(backend.last_link().is_some());

            let link = MockLink::new(TransportDeviceId::new("dev-1"));
            backend.push_link(link);
            let target = ReconnectTarget {
                device_id: TransportDeviceId::new("dev-1"),
                filter,
            };
            let _reconnected = backend.reconnect(&target).await.expect("reconnect");
        });
    }

    #[test]
    fn backend_connect_without_link_fails() {
        block_on(async {
            let backend = MockBackend::new(vec![device("dev-1")]);
            let device = device("dev-1");
            assert!(matches!(
                backend.connect(&device).await,
                Err(TransportError::DeviceNotFound(_))
            ));
        });
    }
}
