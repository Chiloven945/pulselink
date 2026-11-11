//! Coyote V3 GATT service and characteristic identifiers.

use uuid::Uuid;

/// Control service (`0x180C`).
pub const SERVICE_CONTROL: Uuid = uuid::uuid!("0000180c-0000-1000-8000-00805f9b34fb");
/// Write characteristic (`0x150A`).
pub const CHAR_WRITE: Uuid = uuid::uuid!("0000150a-0000-1000-8000-00805f9b34fb");
/// Notify characteristic (`0x150B`).
pub const CHAR_NOTIFY: Uuid = uuid::uuid!("0000150b-0000-1000-8000-00805f9b34fb");
/// Battery service (`0x180A` as used by DG-LAB).
pub const SERVICE_BATTERY: Uuid = uuid::uuid!("0000180a-0000-1000-8000-00805f9b34fb");
/// Battery level characteristic (`0x1500`).
pub const CHAR_BATTERY: Uuid = uuid::uuid!("00001500-0000-1000-8000-00805f9b34fb");

/// Bluetooth name of the Coyote 3.0 stimulator.
pub const DEVICE_NAME: &str = "47L121000";
/// Bluetooth name of the wireless sensor.
pub const SENSOR_NAME: &str = "47L120100";
