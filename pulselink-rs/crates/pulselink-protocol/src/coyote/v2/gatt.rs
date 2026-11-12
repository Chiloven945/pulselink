//! Coyote V2 BLE identifiers.

use uuid::Uuid;

/// PWM service (`0x180B`).
pub const SERVICE_PWM: Uuid = uuid::uuid!("0000180b-0000-1000-8000-00805f9b34fb");
/// A/B intensity characteristic (`0x1504`).
pub const CHAR_PWM_AB2: Uuid = uuid::uuid!("00001504-0000-1000-8000-00805f9b34fb");
/// A-channel pulse characteristic (`0x1505`).
pub const CHAR_PWM_A34: Uuid = uuid::uuid!("00001505-0000-1000-8000-00805f9b34fb");
/// B-channel pulse characteristic (`0x1506`).
pub const CHAR_PWM_B34: Uuid = uuid::uuid!("00001506-0000-1000-8000-00805f9b34fb");
/// Battery service (`0x180A`).
pub const SERVICE_BATTERY: Uuid = uuid::uuid!("0000180a-0000-1000-8000-00805f9b34fb");
/// Battery level characteristic (`0x1500`).
pub const CHAR_BATTERY: Uuid = uuid::uuid!("00001500-0000-1000-8000-00805f9b34fb");
/// Bluetooth name of the Coyote V2 stimulator.
pub const DEVICE_NAME: &str = "D-LAB ESTIM01";
