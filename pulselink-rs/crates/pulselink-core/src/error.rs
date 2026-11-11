//! Errors returned by core value objects.

/// Errors returned when constructing or converting core domain values.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// An intensity value was outside `0..=200`.
    #[error("invalid intensity {0}; expected 0..=200")]
    InvalidIntensity(u16),
    /// An intensity delta was outside `-200..=200`.
    #[error("invalid intensity delta {0}; expected -200..=200")]
    InvalidIntensityDelta(i32),
    /// A battery level was outside `0..=100`.
    #[error("invalid battery level {0}; expected 0..=100")]
    InvalidBatteryLevel(u8),
    /// A channel discriminator was neither `0` (A) nor `1` (B).
    #[error("invalid channel {0}; expected 0 (A) or 1 (B)")]
    InvalidChannel(u8),
}
