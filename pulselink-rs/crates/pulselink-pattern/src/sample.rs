//! Pulse samples and their validated value types.

use std::fmt;
use std::time::Duration;

use crate::error::PatternError;

/// A pulse interval in milliseconds, in the range `10..=1000`.
///
/// The official DG-LAB documentation calls this a "pulse frequency value", but
/// the unit is milliseconds. Use [`PulseInterval::from_hz`] or
/// [`PulseInterval::to_hz`] when working in hertz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16", into = "u16"))]
pub struct PulseInterval(u16);

impl PulseInterval {
    /// The shortest supported interval (10 ms, 100 Hz).
    pub const MIN: Self = Self(10);
    /// The longest supported interval (1000 ms, 1 Hz).
    pub const MAX: Self = Self(1000);

    /// Creates a pulse interval.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidInterval`] when `value` is outside
    /// `10..=1000`.
    pub fn new(value: u16) -> Result<Self, PatternError> {
        if (Self::MIN.0..=Self::MAX.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(PatternError::InvalidInterval(value))
        }
    }

    /// Converts an electrical pulse frequency in hertz to an interval.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidFrequency`] when `hz` is not finite,
    /// non-positive, or maps outside `10..=1000` ms.
    pub fn from_hz(hz: f64) -> Result<Self, PatternError> {
        if !hz.is_finite() || hz <= 0.0 {
            return Err(PatternError::InvalidFrequency);
        }

        let millis = (1000.0 / hz).round();
        if !(10.0..=1000.0).contains(&millis) {
            return Err(PatternError::InvalidFrequency);
        }

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the value was range-checked to 10..=1000 before conversion"
        )]
        let millis = millis as u16;

        Self::new(millis)
    }

    /// Returns the interval in milliseconds.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Returns the interval as a [`Duration`].
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        Duration::from_millis(self.0 as u64)
    }

    /// Returns the approximate electrical pulse frequency in hertz.
    #[must_use]
    pub fn to_hz(self) -> f64 {
        1000.0 / f64::from(self.0)
    }
}

impl TryFrom<u16> for PulseInterval {
    type Error = PatternError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PulseInterval> for u16 {
    fn from(value: PulseInterval) -> Self {
        value.0
    }
}

impl fmt::Display for PulseInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ms", self.0)
    }
}

/// A relative pulse intensity in the range `0..=100`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
pub struct PulseIntensity(u8);

impl PulseIntensity {
    /// The minimum pulse intensity.
    pub const MIN: Self = Self(0);
    /// The maximum pulse intensity.
    pub const MAX: Self = Self(100);

    /// Creates a pulse intensity.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidIntensity`] when `value > 100`.
    pub const fn new(value: u8) -> Result<Self, PatternError> {
        if value <= Self::MAX.0 {
            Ok(Self(value))
        } else {
            Err(PatternError::InvalidIntensity(value))
        }
    }

    /// Returns the raw pulse intensity.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for PulseIntensity {
    type Error = PatternError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PulseIntensity> for u8 {
    fn from(value: PulseIntensity) -> Self {
        value.0
    }
}

impl fmt::Display for PulseIntensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One 25 ms sample of the waveform timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseSample {
    /// Pulse interval for this sample.
    pub interval: PulseInterval,
    /// Pulse intensity for this sample.
    pub intensity: PulseIntensity,
}

impl PulseSample {
    /// Creates a pulse sample.
    #[must_use]
    pub const fn new(interval: PulseInterval, intensity: PulseIntensity) -> Self {
        Self {
            interval,
            intensity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_bounds() {
        assert_eq!(PulseInterval::new(10).map(PulseInterval::get), Ok(10));
        assert_eq!(PulseInterval::new(1000).map(PulseInterval::get), Ok(1000));
        assert!(PulseInterval::new(9).is_err());
        assert!(PulseInterval::new(1001).is_err());
    }

    #[test]
    fn interval_hz_conversion() {
        assert_eq!(
            PulseInterval::from_hz(100.0).map(PulseInterval::get),
            Ok(10)
        );
        assert_eq!(
            PulseInterval::from_hz(1.0).map(PulseInterval::get),
            Ok(1000)
        );
        assert_eq!(PulseInterval::new(100).expect("valid").to_hz(), 10.0);
        assert!(PulseInterval::from_hz(0.0).is_err());
        assert!(PulseInterval::from_hz(f64::NAN).is_err());
        assert!(PulseInterval::from_hz(200.0).is_err());
    }

    #[test]
    fn interval_duration() {
        assert_eq!(
            PulseInterval::new(25).expect("valid").as_duration(),
            Duration::from_millis(25)
        );
        assert_eq!(PulseInterval::new(25).expect("valid").to_string(), "25 ms");
    }

    #[test]
    fn intensity_bounds() {
        assert_eq!(PulseIntensity::new(0).map(PulseIntensity::get), Ok(0));
        assert_eq!(PulseIntensity::new(100).map(PulseIntensity::get), Ok(100));
        assert!(PulseIntensity::new(101).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn sample_serde_validates() {
        let interval = PulseInterval::new(25).expect("valid");
        let intensity = PulseIntensity::new(50).expect("valid");
        let sample = PulseSample::new(interval, intensity);
        let json = serde_json::to_string(&sample).expect("serialize");
        let roundtrip: PulseSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, sample);

        assert!(serde_json::from_str::<PulseInterval>("9").is_err());
        assert!(serde_json::from_str::<PulseIntensity>("101").is_err());
    }
}
