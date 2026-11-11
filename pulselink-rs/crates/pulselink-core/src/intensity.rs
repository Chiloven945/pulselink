//! Intensity values and deltas.

use std::fmt;

use crate::error::CoreError;

/// A logical channel intensity in the range `0..=200`.
///
/// This is the protocol-level intensity used by Coyote V3 and DG-LAB V4. It is
/// not a physical unit and is not a safety recommendation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
pub struct Intensity(u8);

impl Intensity {
    /// The minimum intensity, zero.
    pub const MIN: Self = Self(0);
    /// The maximum protocol intensity.
    pub const MAX: Self = Self(200);

    /// Creates an intensity value.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidIntensity`] when `value > 200`.
    pub fn new(value: u8) -> Result<Self, CoreError> {
        if value <= Self::MAX.0 {
            Ok(Self(value))
        } else {
            Err(CoreError::InvalidIntensity(u16::from(value)))
        }
    }

    /// Returns the raw intensity value.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns whether this is the minimum intensity.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Adds a non-negative delta, returning `None` on overflow past 200.
    #[must_use]
    pub fn checked_add(self, delta: u8) -> Option<Self> {
        let value = self.0.checked_add(delta)?;
        (value <= Self::MAX.0).then_some(Self(value))
    }

    /// Subtracts a non-negative delta, returning `None` on underflow below 0.
    #[must_use]
    pub fn checked_sub(self, delta: u8) -> Option<Self> {
        self.0.checked_sub(delta).map(Self)
    }

    /// Adds a delta, saturating at 200.
    #[must_use]
    pub fn saturating_add(self, delta: u8) -> Self {
        Self(self.0.saturating_add(delta).min(Self::MAX.0))
    }

    /// Subtracts a delta, saturating at 0.
    #[must_use]
    pub fn saturating_sub(self, delta: u8) -> Self {
        Self(self.0.saturating_sub(delta))
    }
}

impl Default for Intensity {
    fn default() -> Self {
        Self::MIN
    }
}

impl TryFrom<u8> for Intensity {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Intensity> for u8 {
    fn from(value: Intensity) -> Self {
        value.0
    }
}

impl fmt::Display for Intensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A signed intensity change in the range `-200..=200`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "i16", into = "i16"))]
pub struct IntensityDelta(i16);

impl IntensityDelta {
    /// The minimum delta.
    pub const MIN: Self = Self(-200);
    /// The maximum delta.
    pub const MAX: Self = Self(200);

    /// Creates an intensity delta.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidIntensityDelta`] when `value` is outside
    /// `-200..=200`.
    pub fn new(value: i16) -> Result<Self, CoreError> {
        if (Self::MIN.0..=Self::MAX.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(CoreError::InvalidIntensityDelta(i32::from(value)))
        }
    }

    /// Returns the raw delta.
    #[must_use]
    pub const fn get(self) -> i16 {
        self.0
    }

    /// Returns whether the delta is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Returns the absolute magnitude of the delta.
    #[must_use]
    pub const fn magnitude(self) -> u16 {
        self.0.unsigned_abs()
    }
}

impl TryFrom<i16> for IntensityDelta {
    type Error = CoreError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<IntensityDelta> for i16 {
    fn from(value: IntensityDelta) -> Self {
        value.0
    }
}

impl fmt::Display for IntensityDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intensity_bounds() {
        assert_eq!(Intensity::new(0).map(Intensity::get), Ok(0));
        assert_eq!(Intensity::new(200).map(Intensity::get), Ok(200));
        assert_eq!(Intensity::new(201), Err(CoreError::InvalidIntensity(201)));
        assert!(Intensity::MIN.is_zero());
        assert!(!Intensity::MAX.is_zero());
    }

    #[test]
    fn intensity_arithmetic() {
        let value = Intensity::new(200).expect("valid");
        assert_eq!(value.checked_add(1), None);
        assert_eq!(value.saturating_add(10), Intensity::MAX);
        let zero = Intensity::MIN;
        assert_eq!(zero.checked_sub(1), None);
        assert_eq!(
            Intensity::new(10)
                .expect("valid")
                .checked_sub(3)
                .map(Intensity::get),
            Some(7)
        );
    }

    #[test]
    fn intensity_delta_bounds() {
        assert_eq!(IntensityDelta::new(-200).map(IntensityDelta::get), Ok(-200));
        assert_eq!(IntensityDelta::new(200).map(IntensityDelta::get), Ok(200));
        assert!(IntensityDelta::new(201).is_err());
        assert!(IntensityDelta::new(-201).is_err());
        assert!(IntensityDelta::new(0).expect("valid").is_zero());
        assert_eq!(IntensityDelta::new(-17).expect("valid").magnitude(), 17);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn intensity_serde_validates() {
        let value = Intensity::new(42).expect("valid");
        let json = serde_json::to_string(&value).expect("serialize");
        assert_eq!(json, "42");
        let roundtrip: Intensity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, value);

        assert!(serde_json::from_str::<Intensity>("201").is_err());
        assert!(serde_json::from_str::<IntensityDelta>("201").is_err());
    }
}
