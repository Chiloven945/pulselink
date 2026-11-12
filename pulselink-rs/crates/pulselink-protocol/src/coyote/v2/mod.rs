//! Coyote V2 BLE protocol.
//!
//! V2 uses three independent PWM characteristics and a 24-bit little-endian
//! layout. It is not a reduced V3 B0 protocol.

mod frequency;
mod gatt;
mod pwm;

pub use frequency::{DEFAULT_PULSE_WIDTH, decode_interval, encode_interval};
pub use gatt::{
    CHAR_BATTERY, CHAR_PWM_A34, CHAR_PWM_AB2, CHAR_PWM_B34, DEVICE_NAME, SERVICE_BATTERY,
    SERVICE_PWM,
};
pub use pwm::{CoyoteV2Intensity, PwmAb2, PwmPulse};
