//! Errors returned by pattern construction and timeline operations.

/// Errors returned by pattern types.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PatternError {
    /// A pattern or timeline must contain at least one frame.
    #[error("pattern is empty")]
    Empty,
    /// A pulse interval was outside `10..=1000` milliseconds.
    #[error("invalid pulse interval {0} ms; expected 10..=1000")]
    InvalidInterval(u16),
    /// A pulse intensity was outside `0..=100`.
    #[error("invalid pulse intensity {0}; expected 0..=100")]
    InvalidIntensity(u8),
    /// A pulse frequency could not be converted to a valid interval.
    #[error("invalid pulse frequency; expected a finite value that maps to 10..=1000 ms")]
    InvalidFrequency,
    /// A frame did not contain exactly four samples.
    #[error("invalid sample count {0}; expected 4")]
    InvalidSampleCount(usize),
    /// A pattern exceeded the supported length.
    #[error("pattern is too long")]
    TooLong,
}
