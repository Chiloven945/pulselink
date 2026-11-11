//! 100 ms stereo frames.

use pulselink_core::Channel;

use crate::sample::PulseSample;

/// The waveform data for one channel in a 100 ms frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChannelFrame {
    /// Four 25 ms samples of active waveform data.
    Active([PulseSample; 4]),
    /// Explicit silence for the whole 100 ms window.
    Silent,
}

impl ChannelFrame {
    /// Creates an active channel frame from four samples.
    #[must_use]
    pub const fn active(samples: [PulseSample; 4]) -> Self {
        Self::Active(samples)
    }

    /// Creates a silent channel frame.
    #[must_use]
    pub const fn silent() -> Self {
        Self::Silent
    }

    /// Returns the active samples, if this frame is not silent.
    #[must_use]
    pub const fn samples(&self) -> Option<&[PulseSample; 4]> {
        match self {
            Self::Active(samples) => Some(samples),
            Self::Silent => None,
        }
    }

    /// Returns whether this frame is silent.
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        matches!(self, Self::Silent)
    }
}

/// A 100 ms frame containing both channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StereoFrame {
    /// Channel A data.
    pub a: ChannelFrame,
    /// Channel B data.
    pub b: ChannelFrame,
}

impl StereoFrame {
    /// Creates a stereo frame.
    #[must_use]
    pub const fn new(a: ChannelFrame, b: ChannelFrame) -> Self {
        Self { a, b }
    }

    /// Creates a frame with both channels silent.
    #[must_use]
    pub const fn silent() -> Self {
        Self {
            a: ChannelFrame::Silent,
            b: ChannelFrame::Silent,
        }
    }

    /// Returns the frame data for a channel.
    #[must_use]
    pub const fn channel(&self, channel: Channel) -> &ChannelFrame {
        match channel {
            Channel::A => &self.a,
            Channel::B => &self.b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::{PulseIntensity, PulseInterval};

    fn sample() -> PulseSample {
        PulseSample::new(
            PulseInterval::new(100).expect("valid interval"),
            PulseIntensity::new(20).expect("valid intensity"),
        )
    }

    #[test]
    fn silent_frame() {
        let frame = StereoFrame::silent();
        assert!(frame.a.is_silent());
        assert!(frame.b.is_silent());
        assert_eq!(frame.channel(Channel::A), &ChannelFrame::Silent);
    }

    #[test]
    fn active_frame() {
        let samples = [sample(); 4];
        let frame = ChannelFrame::active(samples);
        assert!(!frame.is_silent());
        assert_eq!(frame.samples(), Some(&samples));
    }
}
