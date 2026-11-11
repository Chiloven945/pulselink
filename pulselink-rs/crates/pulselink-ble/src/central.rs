//! btleplug adapter selection, scanning, and connection setup.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{
    Central as BtCentral, CentralEvent, Manager as BtManager, Peripheral as BtPeripheral,
    ScanFilter as BtScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use pulselink_transport::{
    Central, Connector, DiscoveredDevice, GattLink, ReconnectTarget, ScanFilter, TransportDeviceId,
    TransportError,
};
use tokio::time::{Instant, timeout};
use tracing::{debug, warn};

use crate::error::BleError;
use crate::link::BtleplugLink;
use crate::scan::{discovered_from_properties, finish_scan, merge_discovered};

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_DISCOVER_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_RECONNECT_SCAN_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Timeouts {
    pub(crate) connect: Duration,
    pub(crate) discover: Duration,
    pub(crate) operation: Duration,
    pub(crate) reconnect_scan: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect: DEFAULT_CONNECT_TIMEOUT,
            discover: DEFAULT_DISCOVER_TIMEOUT,
            operation: DEFAULT_OPERATION_TIMEOUT,
            reconnect_scan: DEFAULT_RECONNECT_SCAN_TIMEOUT,
        }
    }
}

/// A btleplug-backed [`Central`] and [`Connector`].
#[derive(Clone)]
pub struct BtleplugCentral {
    adapter: Adapter,
    timeouts: Timeouts,
    peripherals: Arc<Mutex<HashMap<String, Peripheral>>>,
}

impl BtleplugCentral {
    /// Selects the first available Bluetooth adapter.
    ///
    /// Returns [`BleError::NoAdapter`] when the platform exposes no adapter.
    pub async fn new() -> Result<Self, BleError> {
        Self::new_with_adapter_index(0).await
    }

    /// Selects a Bluetooth adapter by its zero-based index.
    pub async fn new_with_adapter_index(index: usize) -> Result<Self, BleError> {
        let manager = Manager::new().await.map_err(BleError::from_btleplug)?;
        let adapters = manager.adapters().await.map_err(BleError::from_btleplug)?;
        let adapter = adapters.into_iter().nth(index).ok_or(BleError::NoAdapter)?;
        Ok(Self::from_adapter(adapter, Timeouts::default()))
    }

    /// Returns a human-readable description of the selected adapter.
    pub async fn adapter_info(&self) -> Result<String, BleError> {
        self.adapter
            .adapter_info()
            .await
            .map_err(BleError::from_btleplug)
    }

    /// Connects a discovered device and returns the concrete BLE link.
    ///
    /// This is useful for hardware probes that need link-specific inspection.
    pub async fn connect_link(&self, device: &DiscoveredDevice) -> Result<BtleplugLink, BleError> {
        let peripheral = self.peripheral_for(&device.id).await?;
        self.connect_peripheral(peripheral, device.id.clone()).await
    }

    pub(crate) fn from_adapter(adapter: Adapter, timeouts: Timeouts) -> Self {
        Self {
            adapter,
            timeouts,
            peripherals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn peripheral_for(&self, id: &TransportDeviceId) -> Result<Peripheral, BleError> {
        if let Some(peripheral) = self
            .peripherals
            .lock()
            .expect("peripheral cache mutex poisoned")
            .get(id.as_str())
            .cloned()
        {
            return Ok(peripheral);
        }

        let peripherals = self
            .adapter
            .peripherals()
            .await
            .map_err(BleError::from_btleplug)?;
        for peripheral in peripherals {
            let peripheral_id = peripheral.id().to_string();
            self.peripherals
                .lock()
                .expect("peripheral cache mutex poisoned")
                .insert(peripheral_id.clone(), peripheral.clone());
            if peripheral_id == id.as_str() {
                return Ok(peripheral);
            }
        }

        Err(BleError::DeviceNotFound(id.to_string()))
    }

    async fn scan_inner(
        &self,
        filter: ScanFilter,
        scan_timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, BleError> {
        let bt_filter = BtScanFilter {
            services: filter.service_uuids.clone(),
        };
        self.adapter
            .start_scan(bt_filter)
            .await
            .map_err(BleError::from_btleplug)?;
        let guard = ScanGuard::new(self.adapter.clone());

        let mut devices = BTreeMap::new();
        if let Ok(mut events) = self.adapter.events().await {
            let deadline = Instant::now() + scan_timeout;
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }

                let event =
                    match timeout(remaining, futures_util::StreamExt::next(&mut events)).await {
                        Err(_) | Ok(None) => break,
                        Ok(Some(event)) => event,
                    };

                match event {
                    CentralEvent::DeviceDiscovered(id)
                    | CentralEvent::DeviceUpdated(id)
                    | CentralEvent::DeviceServicesModified(id)
                    | CentralEvent::ServicesAdvertisement { id, .. }
                    | CentralEvent::ManufacturerDataAdvertisement { id, .. }
                    | CentralEvent::ServiceDataAdvertisement { id, .. }
                    | CentralEvent::RssiUpdate { id, .. } => {
                        self.record_peripheral(&id, &mut devices).await;
                    }
                    CentralEvent::DeviceConnected(_)
                    | CentralEvent::DeviceDisconnected(_)
                    | CentralEvent::StateUpdate(_) => {}
                }
            }
        } else {
            warn!("could not subscribe to BLE scan events");
        }

        if let Ok(peripherals) = self.adapter.peripherals().await {
            for peripheral in peripherals {
                self.record_peripheral_object(&peripheral, &mut devices)
                    .await;
            }
        }

        guard.stop().await;
        Ok(finish_scan(devices, &filter))
    }

    async fn record_peripheral(
        &self,
        id: &PeripheralId,
        devices: &mut BTreeMap<String, DiscoveredDevice>,
    ) {
        match self.adapter.peripheral(id).await {
            Ok(peripheral) => {
                self.record_peripheral_object(&peripheral, devices).await;
            }
            Err(error) => debug!(%error, "failed to look up scanned peripheral"),
        }
    }

    async fn record_peripheral_object(
        &self,
        peripheral: &Peripheral,
        devices: &mut BTreeMap<String, DiscoveredDevice>,
    ) {
        let id = peripheral.id().to_string();
        let properties = peripheral.properties().await.ok().flatten();
        let discovered = discovered_from_properties(id.clone(), properties.as_ref());
        self.peripherals
            .lock()
            .expect("peripheral cache mutex poisoned")
            .insert(id, peripheral.clone());
        merge_discovered(devices, discovered);
    }

    async fn connect_peripheral(
        &self,
        peripheral: Peripheral,
        device_id: TransportDeviceId,
    ) -> Result<BtleplugLink, BleError> {
        let already_connected = peripheral.is_connected().await.unwrap_or(false);
        if !already_connected {
            peripheral
                .connect_with_timeout(self.timeouts.connect)
                .await
                .map_err(BleError::from_btleplug)?;
        }
        peripheral
            .discover_services_with_timeout(self.timeouts.discover)
            .await
            .map_err(BleError::from_btleplug)?;

        let characteristics = peripheral
            .characteristics()
            .into_iter()
            .map(|characteristic| (characteristic.uuid, characteristic))
            .collect();

        Ok(BtleplugLink::new(
            device_id,
            peripheral,
            characteristics,
            self.timeouts.operation,
        ))
    }
}

#[async_trait]
impl Central for BtleplugCentral {
    async fn scan(
        &self,
        filter: ScanFilter,
        timeout: Duration,
    ) -> Result<Vec<DiscoveredDevice>, TransportError> {
        self.scan_inner(filter, timeout)
            .await
            .map_err(|error| match error {
                BleError::NoAdapter => TransportError::NoAdapter,
                other => TransportError::Scan(other.to_string()),
            })
    }

    async fn connect(
        &self,
        device: &DiscoveredDevice,
    ) -> Result<Box<dyn GattLink>, TransportError> {
        self.connect_link(device)
            .await
            .map(|link| Box::new(link) as Box<dyn GattLink>)
            .map_err(|error| match error {
                BleError::NoAdapter => TransportError::NoAdapter,
                BleError::DeviceNotFound(_) => TransportError::DeviceNotFound(device.id.clone()),
                BleError::Timeout => TransportError::Timeout,
                other => TransportError::Connect(other.to_string()),
            })
    }
}

#[async_trait]
impl Connector for BtleplugCentral {
    async fn reconnect(
        &self,
        target: &ReconnectTarget,
    ) -> Result<Box<dyn GattLink>, TransportError> {
        if let Ok(peripheral) = self.peripheral_for(&target.device_id).await
            && let Ok(link) = self
                .connect_peripheral(peripheral, target.device_id.clone())
                .await
        {
            return Ok(Box::new(link));
        }

        let devices = self
            .scan_inner(target.filter.clone(), self.timeouts.reconnect_scan)
            .await
            .map_err(|error| TransportError::Scan(error.to_string()))?;
        let device = devices
            .iter()
            .find(|device| device.id == target.device_id)
            .or_else(|| devices.first())
            .ok_or_else(|| TransportError::DeviceNotFound(target.device_id.clone()))?;

        let peripheral = self
            .peripheral_for(&device.id)
            .await
            .map_err(|error| TransportError::Connect(error.to_string()))?;
        self.connect_peripheral(peripheral, device.id.clone())
            .await
            .map(|link| Box::new(link) as Box<dyn GattLink>)
            .map_err(|error| match error {
                BleError::Timeout => TransportError::Timeout,
                other => TransportError::Connect(other.to_string()),
            })
    }
}

struct ScanGuard {
    adapter: Adapter,
    active: bool,
}

impl ScanGuard {
    fn new(adapter: Adapter) -> Self {
        Self {
            adapter,
            active: true,
        }
    }

    async fn stop(mut self) {
        self.active = false;
        if let Err(error) = self.adapter.stop_scan().await {
            debug!(%error, "failed to stop BLE scan");
        }
    }
}

impl Drop for ScanGuard {
    fn drop(&mut self) {
        if self.active {
            let adapter = self.adapter.clone();
            tokio::spawn(async move {
                let _ = adapter.stop_scan().await;
            });
        }
    }
}
