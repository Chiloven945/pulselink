//! Coyote V3 battery level decoding.

pub use pulselink_core::BatteryLevel;

use crate::error::ProtocolError;

/// Decodes a 1-byte battery notification.
pub fn decode_battery(bytes: &[u8]) -> Result<BatteryLevel, ProtocolError> {
    if bytes.len() != 1 {
        return Err(ProtocolError::InvalidLength {
            expected: 1,
            actual: bytes.len(),
        });
    }

    BatteryLevel::new(bytes[0]).map_err(|_| ProtocolError::ValueOutOfRange {
        field: "battery level",
        value: i64::from(bytes[0]),
        min: 0,
        max: 100,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_boundaries() {
        assert_eq!(decode_battery(&[0]).map(BatteryLevel::get), Ok(0));
        assert_eq!(decode_battery(&[100]).map(BatteryLevel::get), Ok(100));
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(matches!(
            decode_battery(&[101]),
            Err(ProtocolError::ValueOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_invalid_length() {
        assert!(matches!(
            decode_battery(&[1, 2]),
            Err(ProtocolError::InvalidLength {
                expected: 1,
                actual: 2
            })
        ));
    }
}
