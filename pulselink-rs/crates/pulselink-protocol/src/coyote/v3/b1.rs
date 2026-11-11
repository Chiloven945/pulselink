//! Coyote V3 `B1` intensity response decoding.

use pulselink_core::{ChannelMap, Intensity};

use crate::coyote::v3::b0::Sequence;
use crate::error::ProtocolError;

/// A Coyote V3 `B1` intensity response.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B1Message {
    /// Sequence number from the corresponding B0, or zero for an unsolicited change.
    pub sequence: Sequence,
    /// Actual per-channel intensity.
    pub intensity: ChannelMap<Intensity>,
}

impl B1Message {
    /// Decodes a 4-byte B1 notification.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 4 {
            return Err(ProtocolError::InvalidLength {
                expected: 4,
                actual: bytes.len(),
            });
        }

        if bytes[0] != 0xB1 {
            return Err(ProtocolError::InvalidHeader(bytes[0]));
        }

        let sequence = Sequence::new(bytes[1])?;
        let a = decode_intensity("B1 channel A intensity", bytes[2])?;
        let b = decode_intensity("B1 channel B intensity", bytes[3])?;

        Ok(Self {
            sequence,
            intensity: ChannelMap::new(a, b),
        })
    }
}

fn decode_intensity(field: &'static str, value: u8) -> Result<Intensity, ProtocolError> {
    Intensity::new(value).map_err(|_| ProtocolError::ValueOutOfRange {
        field,
        value: i64::from(value),
        min: 0,
        max: 200,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_sequence_and_intensities() {
        let message = B1Message::decode(&[0xB1, 7, 25, 40]).expect("valid B1");
        assert_eq!(message.sequence, Sequence::new(7).expect("valid sequence"));
        assert_eq!(
            message.intensity.a,
            Intensity::new(25).expect("valid intensity")
        );
        assert_eq!(
            message.intensity.b,
            Intensity::new(40).expect("valid intensity")
        );
    }

    #[test]
    fn sequence_zero_is_unsolicited() {
        let message = B1Message::decode(&[0xB1, 0, 0, 0]).expect("valid B1");
        assert_eq!(message.sequence, Sequence::ZERO);
        assert_eq!(message.intensity.a, Intensity::MIN);
        assert_eq!(message.intensity.b, Intensity::MIN);
    }

    #[test]
    fn rejects_invalid_length() {
        assert!(matches!(
            B1Message::decode(&[0xB1, 1, 2]),
            Err(ProtocolError::InvalidLength {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn rejects_invalid_header() {
        assert!(matches!(
            B1Message::decode(&[0xB0, 1, 2, 3]),
            Err(ProtocolError::InvalidHeader(0xB0))
        ));
    }

    #[test]
    fn rejects_invalid_sequence() {
        assert!(matches!(
            B1Message::decode(&[0xB1, 16, 2, 3]),
            Err(ProtocolError::InvalidSequence(16))
        ));
    }

    #[test]
    fn rejects_invalid_intensity() {
        assert!(matches!(
            B1Message::decode(&[0xB1, 1, 201, 3]),
            Err(ProtocolError::ValueOutOfRange { .. })
        ));
    }
}
