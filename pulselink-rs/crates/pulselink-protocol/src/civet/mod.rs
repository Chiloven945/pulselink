//! Civet V1 pressure sensor BLE protocol.

use pulselink_core::Pressure;
use uuid::Uuid;

use crate::ProtocolError;

/// Control service (`0x180C`).
pub const SERVICE_CONTROL: Uuid = uuid::uuid!("0000180c-0000-1000-8000-00805f9b34fb");
/// Write characteristic (`0x150A`).
pub const CHAR_WRITE: Uuid = uuid::uuid!("0000150a-0000-1000-8000-00805f9b34fb");
/// Notify characteristic (`0x150B`).
pub const CHAR_NOTIFY: Uuid = uuid::uuid!("0000150b-0000-1000-8000-00805f9b34fb");
/// Battery service (`0x180A`).
pub const SERVICE_BATTERY: Uuid = uuid::uuid!("0000180a-0000-1000-8000-00805f9b34fb");
/// Battery characteristic (`0x1500`).
pub const CHAR_BATTERY: Uuid = uuid::uuid!("00001500-0000-1000-8000-00805f9b34fb");
/// Bluetooth name of the Civet sensor.
pub const DEVICE_NAME: &str = "47L124000";

/// Start or stop pressure reporting (`0x50`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Command50 {
    /// Indicator color.
    pub color: u8,
    /// Whether pressure reporting should start.
    pub start: bool,
}

impl Command50 {
    /// Encodes the 17-byte command.
    #[must_use]
    pub fn encode(self) -> [u8; 17] {
        let mut bytes = [0u8; 17];
        bytes[0] = 0x50;
        bytes[1] = self.color;
        bytes[2] = if self.start { 0xD0 } else { 0x00 };
        bytes
    }
}

/// Reset pressure and/or set display orientation (`0x66`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Command66 {
    /// Display orientation, usually `1` or `3`.
    pub orientation: Option<u8>,
    /// Whether to reset the pressure reading.
    pub reset: bool,
}

impl Command66 {
    /// Encodes the 13-byte command.
    pub fn encode(self) -> Result<[u8; 13], ProtocolError> {
        if let Some(orientation) = self.orientation
            && !matches!(orientation, 1 | 3)
        {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Civet orientation",
                value: i64::from(orientation),
                min: 1,
                max: 3,
            });
        }
        let mut bytes = [0u8; 13];
        bytes[0] = 0x66;
        bytes[10] = self.orientation.unwrap_or(0);
        bytes[12] = u8::from(self.reset) * 2;
        Ok(bytes)
    }
}

/// A pressure notification (`0xD0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D0Message {
    /// Indicator color.
    pub color: u8,
    /// Pressure in hundredths of a kilopascal.
    pub pressure: Pressure,
}

impl D0Message {
    /// Decodes a 17-byte pressure message.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 17 {
            return Err(ProtocolError::InvalidLength {
                expected: 17,
                actual: bytes.len(),
            });
        }
        if bytes[0] != 0xD0 {
            return Err(ProtocolError::InvalidHeader(bytes[0]));
        }
        let raw = i16::from_le_bytes([bytes[8], bytes[9]]);
        Ok(Self {
            color: bytes[1],
            pressure: Pressure::from_centi_kpa(i32::from(raw)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_layouts() {
        let start = Command50 {
            color: 1,
            start: true,
        }
        .encode();
        assert_eq!(start.len(), 17);
        assert_eq!(start[0..3], [0x50, 1, 0xD0]);

        let reset = Command66 {
            orientation: None,
            reset: true,
        }
        .encode()
        .expect("reset");
        assert_eq!(reset.len(), 13);
        assert_eq!(reset[0], 0x66);
        assert_eq!(reset[12], 2);
    }

    #[test]
    fn decodes_pressure() {
        let mut bytes = [0u8; 17];
        bytes[0] = 0xD0;
        bytes[8..10].copy_from_slice(&790i16.to_le_bytes());
        assert_eq!(
            D0Message::decode(&bytes).expect("pressure"),
            D0Message {
                color: 0,
                pressure: Pressure::from_centi_kpa(790)
            }
        );
    }
}
