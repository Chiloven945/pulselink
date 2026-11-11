//! Device-agnostic waveform timeline types.
//!
//! This crate models the semantic content of a pulse pattern: 25 ms samples,
//! 100 ms frames, and a timeline. It does not encode Coyote V3 `B0` bytes; that
//! belongs to `pulselink-protocol`.
//!
//! Pulse intervals are expressed in milliseconds. The official DG-LAB documents
//! call this a "pulse frequency value", but its unit is milliseconds: `10 ms`
//! corresponds to `100 Hz`, and `1000 ms` corresponds to `1 Hz`.

mod error;
mod frame;
mod pattern;
mod sample;
mod timeline;

pub use error::PatternError;
pub use frame::{ChannelFrame, StereoFrame};
pub use pattern::{Pattern, PatternCursor};
pub use sample::{PulseIntensity, PulseInterval, PulseSample};
pub use timeline::{FRAME_DURATION, FRAME_DURATION_MS, Timeline};
