//! Coyote V2 24-bit PWM values.

use crate::ProtocolError;

/// A Coyote V2 intensity value.
///
/// The wire range is `0..=2047`; the official app displays a smaller value
/// where one display step corresponds to seven wire units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoyoteV2Intensity(u16);

impl CoyoteV2Intensity {
    /// Minimum intensity.
    pub const MIN: Self = Self(0);
    /// Maximum intensity.
    pub const MAX: Self = Self(2047);

    /// Creates a wire intensity.
    pub const fn new(value: u16) -> Result<Self, ProtocolError> {
        if value <= Self::MAX.0 {
            Ok(Self(value))
        } else {
            Err(ProtocolError::ValueOutOfRange {
                field: "Coyote V2 intensity",
                value: value as i64,
                min: 0,
                max: 2047,
            })
        }
    }

    /// Creates an intensity from the official app display scale (`0..=200`).
    pub fn from_display(value: u8) -> Result<Self, ProtocolError> {
        if value > 200 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Coyote V2 display intensity",
                value: i64::from(value),
                min: 0,
                max: 200,
            });
        }
        Self::new(u16::from(value) * 7)
    }

    /// Returns the wire value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Returns the approximate official-app display value.
    #[must_use]
    pub fn display(self) -> u8 {
        u8::try_from((self.0 / 7).min(200)).expect("display intensity is bounded")
    }
}

/// A/B channel intensity (`PWM_AB2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PwmAb2 {
    /// Channel A intensity.
    pub a: CoyoteV2Intensity,
    /// Channel B intensity.
    pub b: CoyoteV2Intensity,
}

impl PwmAb2 {
    /// Encodes the three-byte little-endian value.
    #[must_use]
    pub fn encode(self) -> [u8; 3] {
        let raw = (u32::from(self.a.get()) << 11) | u32::from(self.b.get());
        let bytes = raw.to_le_bytes();
        [bytes[0], bytes[1], bytes[2]]
    }

    /// Decodes a three-byte value.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 3 {
            return Err(ProtocolError::InvalidLength {
                expected: 3,
                actual: bytes.len(),
            });
        }
        let raw = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
        let a = ((raw >> 11) & 0x7FF) as u16;
        let b = (raw & 0x7FF) as u16;
        Ok(Self {
            a: CoyoteV2Intensity::new(a)?,
            b: CoyoteV2Intensity::new(b)?,
        })
    }
}

/// A V2 pulse parameter set `[X, Y, Z]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PwmPulse {
    /// Consecutive pulse count, `0..=31`.
    pub x: u8,
    /// Idle interval in milliseconds, `0..=1023`.
    pub y: u16,
    /// Pulse width units, `0..=31`; width is `Z * 5 µs`.
    pub z: u8,
}

impl PwmPulse {
    /// Creates a validated pulse parameter set.
    pub fn new(x: u8, y: u16, z: u8) -> Result<Self, ProtocolError> {
        if x > 31 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Coyote V2 X",
                value: i64::from(x),
                min: 0,
                max: 31,
            });
        }
        if y > 1023 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Coyote V2 Y",
                value: i64::from(y),
                min: 0,
                max: 1023,
            });
        }
        if z > 31 {
            return Err(ProtocolError::ValueOutOfRange {
                field: "Coyote V2 Z",
                value: i64::from(z),
                min: 0,
                max: 31,
            });
        }
        Ok(Self { x, y, z })
    }

    /// Encodes the three-byte little-endian value.
    #[must_use]
    pub fn encode(self) -> [u8; 3] {
        let raw = u32::from(self.x) | (u32::from(self.y) << 5) | (u32::from(self.z) << 15);
        let bytes = raw.to_le_bytes();
        [bytes[0], bytes[1], bytes[2]]
    }

    /// Decodes a three-byte value.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 3 {
            return Err(ProtocolError::InvalidLength {
                expected: 3,
                actual: bytes.len(),
            });
        }
        let raw = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
        Self::new(
            (raw & 0x1F) as u8,
            ((raw >> 5) & 0x3FF) as u16,
            ((raw >> 15) & 0x1F) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_pulse_vectors() {
        assert_eq!(
            PwmPulse::new(1, 9, 0).expect("pulse").encode(),
            [0x21, 0x01, 0x00]
        );
        assert_eq!(
            PwmPulse::new(1, 10, 3).expect("pulse").encode(),
            [0x41, 0x81, 0x01]
        );
        assert_eq!(
            PwmPulse::decode(&[0x41, 0x81, 0x01]).expect("decode"),
            PwmPulse::new(1, 10, 3).expect("pulse")
        );
    }

    #[test]
    fn intensity_bit_layout() {
        let value = PwmAb2 {
            a: CoyoteV2Intensity::new(10).expect("a"),
            b: CoyoteV2Intensity::new(20).expect("b"),
        };
        assert_eq!(value.encode(), [0x14, 0x50, 0x00]);
        assert_eq!(PwmAb2::decode(&value.encode()).expect("decode"), value);
    }

    #[test]
    fn bounds_are_validated() {
        assert!(PwmPulse::new(32, 0, 0).is_err());
        assert!(PwmPulse::new(0, 1024, 0).is_err());
        assert!(PwmPulse::new(0, 0, 32).is_err());
        assert!(CoyoteV2Intensity::new(2048).is_err());
        assert_eq!(
            CoyoteV2Intensity::from_display(200).expect("display").get(),
            1400
        );
    }
}
