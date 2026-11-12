//! `PawPrints` V1.1 wireless button sensor BLE protocol.

use uuid::Uuid;

use crate::ProtocolError;

/// Control service (`0x180C`).
pub const SERVICE_CONTROL: Uuid = uuid::uuid!("0000180c-0000-1000-8000-00805f9b34fb");
/// Write characteristic (`0x150A`).
pub const CHAR_WRITE: Uuid = uuid::uuid!("0000150a-0000-1000-8000-00805f9b34fb");
/// Notify characteristic (`0x150B`).
pub const CHAR_NOTIFY: Uuid = uuid::uuid!("0000150b-0000-1000-8000-00805f9b34fb");
/// Bluetooth name of `PawPrints` V1.1.
pub const DEVICE_NAME_V1_1: &str = "47L120300";
/// Bluetooth name of `PawPrints` V1.0.
pub const DEVICE_NAME_V1_0: &str = "47L120100";

const COMMAND_LEN: usize = 17;

/// A 17-byte `0x50` trigger configuration command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Command50 {
    bytes: [u8; COMMAND_LEN],
}

impl Command50 {
    /// Creates a no-trigger configuration, required immediately after connecting.
    #[must_use]
    pub fn no_trigger(color: u8) -> Self {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = 0x50;
        bytes[1] = color;
        Self { bytes }
    }

    /// Creates a random-reaction trigger configuration.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn random_reaction(
        color: u8,
        event_id: u8,
        delay_range: (u16, u16),
        reaction_seconds: u16,
        increase_rate: u8,
        decrease_on_deactivation: u8,
        decrease_rate: u8,
        increase_on_activation: u8,
    ) -> Self {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = 0x50;
        bytes[1] = color;
        bytes[2] = 0x03;
        bytes[3] = event_id;
        bytes[4..6].copy_from_slice(&delay_range.0.to_le_bytes());
        bytes[6..8].copy_from_slice(&delay_range.1.to_le_bytes());
        bytes[8..10].copy_from_slice(&reaction_seconds.to_le_bytes());
        bytes[10] = increase_rate;
        bytes[11] = decrease_on_deactivation;
        bytes[12] = decrease_rate;
        bytes[13] = increase_on_activation;
        Self { bytes }
    }

    /// Creates a probability trigger configuration.
    ///
    /// Each pair is `(event_id, probability)`; unused pairs should use `(0, 0)`.
    #[must_use]
    pub fn probability(color: u8, events: [(u8, u8); 6], cooldown_seconds: u16) -> Self {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = 0x50;
        bytes[1] = color;
        bytes[2] = 0x04;
        for (index, (event, probability)) in events.iter().enumerate() {
            bytes[3 + index * 2] = *event;
            bytes[4 + index * 2] = *probability;
        }
        bytes[15..17].copy_from_slice(&cooldown_seconds.to_le_bytes());
        Self { bytes }
    }

    /// Creates an external-voltage trigger configuration.
    #[must_use]
    pub fn external_voltage(
        color: u8,
        event_id: u8,
        high_impedance: bool,
        target_range: (u8, u8),
        mapping_range: u8,
    ) -> Self {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = 0x50;
        bytes[1] = color;
        bytes[2] = 0x0F;
        bytes[3] = event_id;
        bytes[4] = u8::from(high_impedance);
        bytes[5] = target_range.0;
        bytes[6] = target_range.1;
        bytes[7] = mapping_range;
        Self { bytes }
    }

    /// Enables raw sensor-data mode.
    #[must_use]
    pub fn raw_sensor(color: u8) -> Self {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = 0x50;
        bytes[1] = color;
        bytes[2] = 0xD0;
        bytes[3] = 0x01;
        Self { bytes }
    }

    /// Returns the raw command bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; COMMAND_LEN] {
        &self.bytes
    }
}

/// Reset the current trigger parameter (`0x5F`).
pub const COMMAND_RESET_PARAMETER: [u8; 1] = [0x5F];
/// Start automatic angle detection (`0x60`).
pub const COMMAND_AUTO_DETECT_ANGLES: [u8; 1] = [0x60];

/// Set a steady shoulder-light color (`0x70`).
#[must_use]
pub fn command_shoulder_light(color: u8) -> [u8; 2] {
    [0x70, color]
}

/// Configure flashing shoulder lights (`0x70`).
#[must_use]
pub fn command_shoulder_light_flash(color_a: u8, color_b: u8, speed: u8) -> [u8; 4] {
    [0x70, color_a, color_b, speed]
}

/// Device information notification (`0x51`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Message51 {
    /// Main indicator color.
    pub color: u8,
    /// Device type; `PawPrints` V1.1 is `0x03`.
    pub device_type: u8,
    /// Battery level `0..=100`.
    pub battery: u8,
}

/// Trigger activated (`0x5A`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Message5a {
    /// Main indicator color.
    pub color: u8,
    /// Trigger event id.
    pub event_id: u8,
    /// Trigger parameter.
    pub parameter: u8,
}

/// Trigger deactivated (`0x5B`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Message5b {
    /// Main indicator color.
    pub color: u8,
    /// Trigger event id.
    pub event_id: u8,
}

/// Trigger parameter changed (`0x5C`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Message5c {
    /// Main indicator color.
    pub color: u8,
    /// Trigger event id.
    pub event_id: u8,
    /// New parameter.
    pub parameter: u8,
}

/// Raw sensor data (`0xD0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D0Message {
    /// Main indicator color.
    pub color: u8,
    /// Sequence number.
    pub sequence: u8,
    /// Button state; nonzero means pressed.
    pub button: u8,
    /// Acceleration.
    pub acceleration: u8,
    /// X angle, signed.
    pub x: i8,
    /// Y angle, signed.
    pub y: i8,
    /// Z angle, signed.
    pub z: i8,
    /// External voltage.
    pub voltage: u8,
}

/// Automatically detected angle thresholds (`0xF1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct F1Message {
    /// X lower/upper bounds after dividing by two.
    pub x: (i8, i8),
    /// Y lower/upper bounds after dividing by two.
    pub y: (i8, i8),
    /// Z lower/upper bounds after dividing by two.
    pub z: (i8, i8),
}

/// A decoded `PawPrints` notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    /// Device information.
    Info(Message51),
    /// Trigger activated.
    Activated(Message5a),
    /// Trigger deactivated.
    Deactivated(Message5b),
    /// Trigger parameter changed.
    Parameter(Message5c),
    /// Raw sensor data.
    Sensor(D0Message),
    /// Auto-detected angle thresholds.
    Angles(F1Message),
}

impl Message {
    /// Decodes a notification by its header.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match bytes.first().copied() {
            Some(0x51) if bytes.len() == 4 => Ok(Self::Info(Message51 {
                color: bytes[1],
                device_type: bytes[2],
                battery: bytes[3],
            })),
            Some(0x5A) if bytes.len() == 4 => Ok(Self::Activated(Message5a {
                color: bytes[1],
                event_id: bytes[2],
                parameter: bytes[3],
            })),
            Some(0x5B) if bytes.len() == 3 => Ok(Self::Deactivated(Message5b {
                color: bytes[1],
                event_id: bytes[2],
            })),
            Some(0x5C) if bytes.len() == 4 => Ok(Self::Parameter(Message5c {
                color: bytes[1],
                event_id: bytes[2],
                parameter: bytes[3],
            })),
            Some(0xD0) if bytes.len() == 9 => Ok(Self::Sensor(D0Message {
                color: bytes[1],
                sequence: bytes[2],
                button: bytes[3],
                acceleration: bytes[4],
                x: bytes[5].cast_signed(),
                y: bytes[6].cast_signed(),
                z: bytes[7].cast_signed(),
                voltage: bytes[8],
            })),
            Some(0xF1) if bytes.len() == 14 && bytes[1] == 0x61 => {
                let pair = |offset: usize| -> (i8, i8) {
                    let lower = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]) / 2;
                    let upper = i16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]) / 2;
                    (
                        i8::try_from(lower.clamp(-128, 127)).expect("clamped i8"),
                        i8::try_from(upper.clamp(-128, 127)).expect("clamped i8"),
                    )
                };
                Ok(Self::Angles(F1Message {
                    x: pair(2),
                    y: pair(6),
                    z: pair(10),
                }))
            }
            Some(header) => Err(ProtocolError::InvalidHeader(header)),
            None => Err(ProtocolError::InvalidLength {
                expected: 4,
                actual: 0,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_trigger_layout() {
        let command = Command50::no_trigger(1);
        let bytes = command.as_bytes();
        assert_eq!(bytes.len(), 17);
        assert_eq!(&bytes[..3], &[0x50, 1, 0x00]);
    }

    #[test]
    fn probability_layout() {
        let command =
            Command50::probability(1, [(5, 60), (2, 80), (0, 0), (0, 0), (0, 0), (0, 0)], 600);
        assert_eq!(&command.as_bytes()[2..6], &[0x04, 5, 60, 2]);
        assert_eq!(command.as_bytes()[15..17], 600u16.to_le_bytes());
    }

    #[test]
    fn decodes_trigger_messages() {
        assert!(matches!(
            Message::decode(&[0x5A, 1, 7, 42]).expect("5A"),
            Message::Activated(Message5a {
                event_id: 7,
                parameter: 42,
                ..
            })
        ));
        let sensor = Message::decode(&[0xD0, 1, 7, 1, 10, 0xFF, 2, 3, 9]).expect("D0");
        assert!(matches!(sensor, Message::Sensor(D0Message { x: -1, .. })));
    }
}
