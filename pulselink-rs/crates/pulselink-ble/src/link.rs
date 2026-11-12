//! A [`GattLink`] backed by a btleplug peripheral.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{CharPropFlags, Characteristic, Peripheral as BtPeripheral, WriteType};
use btleplug::platform::Peripheral;
use pulselink_transport::{
    GattLink, NotificationStream, TransportDeviceId, TransportError, WriteMode,
};
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

use crate::error::BleError;
use crate::notify::NotificationHub;

/// Capabilities of a discovered GATT characteristic.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacteristicInfo {
    /// The characteristic supports reads.
    pub readable: bool,
    /// The characteristic supports writes with a response.
    pub writable: bool,
    /// The characteristic supports writes without a response.
    pub writable_without_response: bool,
    /// The characteristic supports notifications.
    pub notifiable: bool,
    /// The characteristic supports indications.
    pub indicatable: bool,
}

impl CharacteristicInfo {
    fn from_flags(flags: CharPropFlags) -> Self {
        Self {
            readable: flags.contains(CharPropFlags::READ),
            writable: flags.contains(CharPropFlags::WRITE),
            writable_without_response: flags.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE),
            notifiable: flags.contains(CharPropFlags::NOTIFY),
            indicatable: flags.contains(CharPropFlags::INDICATE),
        }
    }
}

/// A connected btleplug peripheral implementing [`GattLink`].
pub struct BtleplugLink {
    device_id: TransportDeviceId,
    peripheral: Peripheral,
    characteristics: HashMap<Uuid, Characteristic>,
    connected: AtomicBool,
    write_lock: Mutex<()>,
    notification_hub: Arc<NotificationHub>,
    operation_timeout: Duration,
}

impl BtleplugLink {
    pub(crate) fn new(
        device_id: TransportDeviceId,
        peripheral: Peripheral,
        characteristics: HashMap<Uuid, Characteristic>,
        operation_timeout: Duration,
    ) -> Self {
        Self {
            device_id,
            peripheral,
            characteristics,
            connected: AtomicBool::new(true),
            write_lock: Mutex::new(()),
            notification_hub: Arc::new(NotificationHub::default()),
            operation_timeout,
        }
    }

    /// Returns the capabilities of a discovered characteristic.
    #[must_use]
    pub fn characteristic(&self, characteristic: Uuid) -> Option<CharacteristicInfo> {
        self.characteristics
            .get(&characteristic)
            .map(|characteristic| CharacteristicInfo::from_flags(characteristic.properties))
    }

    /// Returns the UUIDs of all discovered characteristics.
    #[must_use]
    pub fn characteristic_uuids(&self) -> Vec<Uuid> {
        let mut uuids: Vec<_> = self.characteristics.keys().copied().collect();
        uuids.sort_unstable();
        uuids
    }

    fn ensure_connected(&self) -> Result<(), TransportError> {
        if self.connected.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(TransportError::Disconnected)
        }
    }

    fn characteristic_or_error(
        &self,
        characteristic: Uuid,
    ) -> Result<&Characteristic, TransportError> {
        self.characteristics.get(&characteristic).ok_or_else(|| {
            TransportError::Gatt(format!("characteristic not found: {characteristic}"))
        })
    }

    fn mark_disconnected(&self) {
        self.connected.store(false, Ordering::SeqCst);
    }

    fn map_gatt_error(&self, error: btleplug::Error) -> TransportError {
        match error {
            btleplug::Error::NotConnected => {
                self.mark_disconnected();
                TransportError::Disconnected
            }
            other => TransportError::Gatt(other.to_string()),
        }
    }
}

#[async_trait]
impl GattLink for BtleplugLink {
    fn device_id(&self) -> &TransportDeviceId {
        &self.device_id
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    fn mtu(&self) -> Option<u16> {
        let mtu = self.peripheral.mtu();
        (mtu > 0).then_some(mtu)
    }

    async fn write(
        &self,
        characteristic: Uuid,
        data: &[u8],
        mode: WriteMode,
    ) -> Result<(), TransportError> {
        let _guard = self.write_lock.lock().await;
        self.ensure_connected()?;
        let characteristic = self.characteristic_or_error(characteristic)?;
        let write_type = select_write_type(characteristic.properties, mode)
            .map_err(|error| TransportError::Gatt(error.to_string()))?;

        timeout(
            self.operation_timeout,
            self.peripheral.write(characteristic, data, write_type),
        )
        .await
        .map_err(|_| TransportError::Timeout)?
        .map_err(|error| self.map_gatt_error(error))
    }

    async fn read(&self, characteristic: Uuid) -> Result<Vec<u8>, TransportError> {
        self.ensure_connected()?;
        let characteristic = self.characteristic_or_error(characteristic)?;
        if !characteristic.properties.contains(CharPropFlags::READ) {
            return Err(TransportError::Gatt(
                "characteristic does not support read".to_owned(),
            ));
        }

        timeout(self.operation_timeout, self.peripheral.read(characteristic))
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|error| self.map_gatt_error(error))
    }

    async fn subscribe(&self, characteristic: Uuid) -> Result<NotificationStream, TransportError> {
        self.ensure_connected()?;
        let characteristic = self.characteristic_or_error(characteristic)?;
        if !characteristic
            .properties
            .intersects(CharPropFlags::NOTIFY | CharPropFlags::INDICATE)
        {
            return Err(TransportError::Notify(
                "characteristic does not support notify or indicate".to_owned(),
            ));
        }

        self.notification_hub
            .subscribe(&self.peripheral, characteristic, self.operation_timeout)
            .await
    }

    async fn disconnect(&self) -> Result<(), TransportError> {
        self.notification_hub.shutdown();
        self.mark_disconnected();

        match timeout(self.operation_timeout, self.peripheral.disconnect()).await {
            Err(_) => Err(TransportError::Timeout),
            Ok(Ok(()) | Err(btleplug::Error::NotConnected)) => Ok(()),
            Ok(Err(error)) => Err(TransportError::Gatt(error.to_string())),
        }
    }
}

impl Drop for BtleplugLink {
    fn drop(&mut self) {
        self.mark_disconnected();
        self.notification_hub.shutdown();

        let peripheral = self.peripheral.clone();
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                let _ = peripheral.disconnect().await;
            });
        }
    }
}

fn select_write_type(properties: CharPropFlags, mode: WriteMode) -> Result<WriteType, BleError> {
    match mode {
        WriteMode::Auto => {
            if properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                Ok(WriteType::WithoutResponse)
            } else if properties.contains(CharPropFlags::WRITE) {
                Ok(WriteType::WithResponse)
            } else {
                Err(BleError::Gatt(
                    "characteristic does not support writes".to_owned(),
                ))
            }
        }
        WriteMode::WithResponse => {
            if properties.contains(CharPropFlags::WRITE) {
                Ok(WriteType::WithResponse)
            } else {
                Err(BleError::Gatt(
                    "characteristic does not support write with response".to_owned(),
                ))
            }
        }
        WriteMode::WithoutResponse => {
            if properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                Ok(WriteType::WithoutResponse)
            } else {
                Err(BleError::Gatt(
                    "characteristic does not support write without response".to_owned(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_prefers_without_response() {
        let properties = CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE;
        assert_eq!(
            select_write_type(properties, WriteMode::Auto).expect("write type"),
            WriteType::WithoutResponse
        );
    }

    #[test]
    fn auto_falls_back_to_with_response() {
        assert_eq!(
            select_write_type(CharPropFlags::WRITE, WriteMode::Auto).expect("write type"),
            WriteType::WithResponse
        );
    }

    #[test]
    fn explicit_modes_require_matching_properties() {
        assert!(select_write_type(CharPropFlags::WRITE, WriteMode::WithoutResponse).is_err());
        assert!(
            select_write_type(
                CharPropFlags::WRITE_WITHOUT_RESPONSE,
                WriteMode::WithResponse
            )
            .is_err()
        );
        assert!(select_write_type(CharPropFlags::READ, WriteMode::Auto).is_err());
    }

    #[test]
    fn characteristic_info_maps_flags() {
        let info = CharacteristicInfo::from_flags(
            CharPropFlags::READ | CharPropFlags::WRITE | CharPropFlags::NOTIFY,
        );
        assert!(info.readable);
        assert!(info.writable);
        assert!(info.notifiable);
        assert!(!info.writable_without_response);
        assert!(!info.indicatable);
    }
}
