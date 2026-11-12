//! Opossum V1 vibration controller BLE protocol.

use pulselink_core::ButtonMask;
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
/// Bluetooth name of the Opossum controller.
pub const DEVICE_NAME: &str = "47L127000";

const B2_FIXED: [u8; 21] = [
    0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0x08, 0x09,
];

/// Indicator color and button-reporting command (`0x50`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Command50 {
    /// Indicator color byte.
    pub color: u8,
    /// Whether physical button reports are enabled.
    pub button_report: bool,
}

impl Command50 {
    /// Encodes the command.
    #[must_use]
    pub fn encode(self) -> [u8; 3] {
        [0x50, self.color, u8::from(self.button_report)]
    }
}

/// Dual-channel vibration frame (`0xB0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B0 {
    /// Channel A four 25 ms intensities.
    pub a: [u8; 4],
    /// Channel B four 25 ms intensities.
    pub b: [u8; 4],
}

impl B0 {
    /// Validates and encodes the 20-byte command.
    pub fn encode(self) -> Result<[u8; 20], ProtocolError> {
        if self.a.iter().chain(self.b.iter()).any(|value| *value > 100) {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Opossum vibration intensity",
                value: 101,
                min: 0,
                max: 100,
            });
        }
        let mut bytes = [0u8; 20];
        bytes[0] = 0xB0;
        bytes[8..12].copy_from_slice(&self.a);
        bytes[16..20].copy_from_slice(&self.b);
        Ok(bytes)
    }
}

/// Channel intensity command (`0xB3`); `None` leaves the channel unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B3 {
    /// Channel A intensity or unchanged.
    pub a: Option<u8>,
    /// Channel B intensity or unchanged.
    pub b: Option<u8>,
}

impl B3 {
    /// Encodes the command.
    pub fn encode(self) -> Result<[u8; 3], ProtocolError> {
        let validate = |value: Option<u8>| -> Result<u8, ProtocolError> {
            match value {
                Some(value) if value <= 200 => Ok(value),
                Some(value) => Err(ProtocolError::ValueOutOfRange {
                    field: "Opossum intensity",
                    value: i64::from(value),
                    min: 0,
                    max: 200,
                }),
                None => Ok(0xFF),
            }
        };
        Ok([0xB3, validate(self.a)?, validate(self.b)?])
    }
}

/// Display intensity sync command (`0xB2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B2 {
    /// Channel A intensity shown on the display.
    pub a: u8,
    /// Channel B intensity shown on the display.
    pub b: u8,
}

impl B2 {
    /// Encodes the 24-byte command.
    pub fn encode(self) -> Result<[u8; 24], ProtocolError> {
        if self.a > 200 || self.b > 200 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Opossum display intensity",
                value: i64::from(self.a.max(self.b)),
                min: 0,
                max: 200,
            });
        }
        let mut bytes = [0u8; 24];
        bytes[0] = 0xB2;
        bytes[1..22].copy_from_slice(&B2_FIXED);
        bytes[22] = self.a;
        bytes[23] = self.b;
        Ok(bytes)
    }
}

/// Intensity notification (`0xB3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B3Message {
    /// Channel A intensity.
    pub a: u8,
    /// Channel B intensity.
    pub b: u8,
}

impl B3Message {
    /// Decodes a three-byte message.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 3 {
            return Err(ProtocolError::InvalidLength {
                expected: 3,
                actual: bytes.len(),
            });
        }
        if bytes[0] != 0xB3 {
            return Err(ProtocolError::InvalidHeader(bytes[0]));
        }
        if bytes[1] > 200 || bytes[2] > 200 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Opossum intensity notification",
                value: i64::from(bytes[1].max(bytes[2])),
                min: 0,
                max: 200,
            });
        }
        Ok(Self {
            a: bytes[1],
            b: bytes[2],
        })
    }
}

/// Button notification (`0xD0`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct D0Message {
    /// Sequence number.
    pub sequence: u8,
    /// Button-state bitmask.
    pub buttons: ButtonMask,
    /// Full 16-byte message including the header.
    pub raw: [u8; 16],
}

impl D0Message {
    /// Decodes a 16-byte message.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 16 {
            return Err(ProtocolError::InvalidLength {
                expected: 16,
                actual: bytes.len(),
            });
        }
        if bytes[0] != 0xD0 {
            return Err(ProtocolError::InvalidHeader(bytes[0]));
        }
        let mut raw = [0u8; 16];
        raw.copy_from_slice(bytes);
        let buttons = ButtonMask::from_bits(u16::from_le_bytes([bytes[2], bytes[3]]));
        Ok(Self {
            sequence: bytes[1],
            buttons,
            raw,
        })
    }
}

/// A decoded Opossum notification.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    /// Intensity update.
    B3(B3Message),
    /// Button update.
    D0(D0Message),
}

impl Message {
    /// Decodes a notification by its header.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match bytes.first().copied() {
            Some(0xB3) => B3Message::decode(bytes).map(Self::B3),
            Some(0xD0) => D0Message::decode(bytes).map(Self::D0),
            Some(header) => Err(ProtocolError::InvalidHeader(header)),
            None => Err(ProtocolError::InvalidLength {
                expected: 3,
                actual: 0,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b0_layout() {
        let bytes = B0 {
            a: [0, 25, 50, 100],
            b: [100, 50, 25, 0],
        }
        .encode()
        .expect("B0");
        assert_eq!(bytes.len(), 20);
        assert_eq!(&bytes[8..12], &[0, 25, 50, 100]);
        assert_eq!(&bytes[16..20], &[100, 50, 25, 0]);
    }

    #[test]
    fn b3_and_b2() {
        assert_eq!(
            B3 {
                a: Some(160),
                b: None
            }
            .encode()
            .expect("B3"),
            [0xB3, 160, 0xFF]
        );
        let b2 = B2 { a: 10, b: 20 }.encode().expect("B2");
        assert_eq!(b2.len(), 24);
        assert_eq!(b2[22..], [10, 20]);
    }

    #[test]
    fn decodes_messages() {
        assert_eq!(
            Message::decode(&[0xB3, 10, 20]).expect("B3"),
            Message::B3(B3Message { a: 10, b: 20 })
        );
        let d0 = Message::decode(&[0xD0, 7, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).expect("D0");
        assert!(
            matches!(d0, Message::D0(message) if message.sequence == 7 && message.buttons.bits() == 2)
        );
    }
}
