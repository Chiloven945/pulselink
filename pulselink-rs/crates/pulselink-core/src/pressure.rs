//! Pressure values used by sensor devices.

/// A pressure reading in hundredths of a kilopascal (0.01 kPa).
///
/// Civet reports pressure as `i16 / 100` kPa. Keeping a fixed-point value makes
/// the type comparable and hashable without floating-point equality issues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pressure(i32);

impl Pressure {
    /// Creates a pressure from hundredths of a kilopascal.
    #[must_use]
    pub const fn from_centi_kpa(value: i32) -> Self {
        Self(value)
    }

    /// Returns the pressure in hundredths of a kilopascal.
    #[must_use]
    pub const fn as_centi_kpa(self) -> i32 {
        self.0
    }

    /// Creates a pressure from kilopascals, rounded to 0.01 kPa.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_kpa(value: f32) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let scaled = (value * 100.0).round();
        if scaled < i32::MIN as f32 || scaled > i32::MAX as f32 {
            None
        } else {
            #[allow(clippy::cast_possible_truncation)]
            Some(Self(scaled as i32))
        }
    }

    /// Creates a pressure from kilopascals, rounded to 0.01 kPa.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn from_kpa_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let scaled = (value * 100.0).round();
        if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
            None
        } else {
            Some(Self(scaled as i32))
        }
    }

    /// Returns the pressure in kilopascals.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_kpa(self) -> f32 {
        self.0 as f32 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centi_kpa_round_trip() {
        let pressure = Pressure::from_centi_kpa(790);
        assert_eq!(pressure.as_centi_kpa(), 790);
        assert_eq!(pressure.as_kpa(), 7.9);
    }

    #[test]
    fn kpa_conversion_rounds() {
        assert_eq!(
            Pressure::from_kpa(17.04),
            Some(Pressure::from_centi_kpa(1704))
        );
        assert_eq!(Pressure::from_kpa(f32::NAN), None);
        assert_eq!(Pressure::from_kpa(f32::INFINITY), None);
    }
}
