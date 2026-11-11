//! Pulse interval to Coyote V3 wire-frequency conversion.
//!
//! The official "pulse frequency value" is in milliseconds. The V3 wire value
//! is compressed into `10..=240`.

use pulselink_pattern::PulseInterval;

use crate::error::ProtocolError;

/// Converts a pulse interval to the Coyote V3 wire value.
///
/// [`PulseInterval`] guarantees `10..=1000` ms, so the result is always in
/// `10..=240`.
#[must_use]
pub fn encode_interval(interval: PulseInterval) -> u8 {
    let millis = interval.get();

    #[allow(
        clippy::cast_possible_truncation,
        reason = "PulseInterval is 10..=1000 and the mapped values are 10..=240"
    )]
    match millis {
        10..=100 => millis as u8,
        101..=600 => ((millis - 100) / 5 + 100) as u8,
        601..=1000 => ((millis - 600) / 10 + 200) as u8,
        _ => 10,
    }
}

/// Converts a Coyote V3 wire value back to a representative pulse interval.
///
/// The reverse mapping is not unique because multiple milliseconds values map
/// to the same wire value. This function returns the lower bound of the
/// corresponding input range. It is suitable for display and diagnostics, not
/// for reconstructing the exact original interval.
pub fn decode_wire_frequency(value: u8) -> Result<PulseInterval, ProtocolError> {
    let millis = match value {
        10..=100 => u16::from(value),
        101..=200 => 100 + (u16::from(value) - 100) * 5,
        201..=240 => 600 + (u16::from(value) - 200) * 10,
        _ => {
            return Err(ProtocolError::ValueOutOfRange {
                field: "wire frequency",
                value: i64::from(value),
                min: 10,
                max: 240,
            });
        }
    };

    PulseInterval::new(millis).map_err(|_| ProtocolError::ValueOutOfRange {
        field: "representative interval",
        value: i64::from(millis),
        min: 10,
        max: 1000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interval(value: u16) -> PulseInterval {
        PulseInterval::new(value).expect("valid interval")
    }

    #[test]
    fn forward_mapping_matches_official_table() {
        let cases = [
            (10, 10),
            (100, 100),
            (110, 102),
            (150, 110),
            (600, 200),
            (650, 205),
            (1000, 240),
        ];

        for (millis, wire) in cases {
            assert_eq!(encode_interval(interval(millis)), wire, "millis={millis}");
        }
    }

    #[test]
    fn boundary_values() {
        assert_eq!(encode_interval(interval(10)), 10);
        assert_eq!(encode_interval(interval(100)), 100);
        assert_eq!(encode_interval(interval(101)), 100);
        assert_eq!(encode_interval(interval(600)), 200);
        assert_eq!(encode_interval(interval(601)), 200);
        assert_eq!(encode_interval(interval(1000)), 240);
    }

    #[test]
    fn reverse_mapping_returns_representative_values() {
        assert_eq!(decode_wire_frequency(10).map(PulseInterval::get), Ok(10));
        assert_eq!(decode_wire_frequency(100).map(PulseInterval::get), Ok(100));
        assert_eq!(decode_wire_frequency(101).map(PulseInterval::get), Ok(105));
        assert_eq!(decode_wire_frequency(200).map(PulseInterval::get), Ok(600));
        assert_eq!(decode_wire_frequency(201).map(PulseInterval::get), Ok(610));
        assert_eq!(decode_wire_frequency(240).map(PulseInterval::get), Ok(1000));
    }

    #[test]
    fn reverse_mapping_rejects_invalid_wire_values() {
        assert!(decode_wire_frequency(9).is_err());
        assert!(decode_wire_frequency(241).is_err());
    }

    #[test]
    fn reverse_mapping_is_valid_for_every_wire_value() {
        for wire in 10..=240 {
            let interval = decode_wire_frequency(wire).expect("valid wire value");
            assert!((10..=1000).contains(&interval.get()));
        }
    }
}
