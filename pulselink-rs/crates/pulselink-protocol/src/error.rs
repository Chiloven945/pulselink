//! Protocol errors shared by Coyote V3 and DG-LAB V4 codecs.

/// Errors returned by protocol encoding, decoding, and validation.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ProtocolError {
    /// A frame had an unexpected length.
    #[error("invalid frame length: expected {expected}, got {actual}")]
    InvalidLength {
        /// Expected byte length.
        expected: usize,
        /// Actual byte length.
        actual: usize,
    },
    /// A frame started with an unexpected header byte.
    #[error("invalid header: 0x{0:02X}")]
    InvalidHeader(u8),
    /// A sequence number was outside `0..=15`.
    #[error("invalid sequence {0}; expected 0..=15")]
    InvalidSequence(u8),
    /// A numeric value was outside its allowed range.
    #[error("value out of range for {field}: {value} (expected {min}..={max})")]
    ValueOutOfRange {
        /// Field name.
        field: &'static str,
        /// Actual value.
        value: i64,
        /// Minimum allowed value.
        min: i64,
        /// Maximum allowed value.
        max: i64,
    },
    /// A pulse frame was structurally invalid.
    #[error("invalid pulse frame: {0}")]
    InvalidPulseFrame(&'static str),
    /// A V4 action type was not one of the supported values.
    #[error("unsupported V4 action type: {0}")]
    UnsupportedActionType(u8),
    /// A cross-field or type-specific parameter was invalid.
    #[error("invalid parameters: {0}")]
    InvalidParameters(&'static str),
    /// A JSON value could not be parsed.
    #[error("invalid JSON: {0}")]
    Json(String),
}

#[cfg(feature = "socket-v4")]
impl From<serde_json::Error> for ProtocolError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}
