//! Interval to Coyote V2 pulse conversion.

use pulselink_pattern::PulseInterval;

use crate::ProtocolError;

use super::pwm::PwmPulse;

/// Default pulse width parameter; `20` corresponds to 100 µs.
pub const DEFAULT_PULSE_WIDTH: u8 = 20;

/// Converts a pulse interval to the recommended V2 `[X, Y, Z]` values.
#[must_use]
pub fn encode_interval(interval: PulseInterval, width: u8) -> PwmPulse {
    let frequency = interval.get();
    let suggested = (f64::from(frequency) / 1000.0).sqrt() * 15.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let x = (suggested.floor() as u8).clamp(1, 31);
    let y = frequency.saturating_sub(u16::from(x)).min(1023);
    PwmPulse {
        x,
        y,
        z: width.min(31),
    }
}

/// Decodes a V2 pulse into its representative interval in milliseconds.
pub fn decode_interval(pulse: PwmPulse) -> Result<PulseInterval, ProtocolError> {
    let interval = u16::from(pulse.x) + pulse.y;
    PulseInterval::new(interval).map_err(|_| ProtocolError::ValueOutOfRange {
        field: "Coyote V2 pulse interval",
        value: i64::from(interval),
        min: 10,
        max: 1000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_breathing_example_maps_to_interval() {
        let pulse = encode_interval(PulseInterval::new(10).expect("interval"), 0);
        assert_eq!(pulse.x, 1);
        assert_eq!(pulse.y, 9);
        assert_eq!(decode_interval(pulse).expect("interval").get(), 10);
    }

    #[test]
    fn width_is_clamped() {
        let pulse = encode_interval(PulseInterval::new(100).expect("interval"), 40);
        assert_eq!(pulse.z, 31);
    }
}
