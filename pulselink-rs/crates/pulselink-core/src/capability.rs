//! Device capability descriptions.

use std::time::Duration;

use crate::channel::Channel;
use crate::intensity::Intensity;

/// The set of channels a device exposes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelSet {
    /// Whether channel A is present.
    pub a: bool,
    /// Whether channel B is present.
    pub b: bool,
}

impl ChannelSet {
    /// No channels.
    pub const NONE: Self = Self { a: false, b: false };
    /// Channel A only.
    pub const A: Self = Self { a: true, b: false };
    /// Channel B only.
    pub const B: Self = Self { a: false, b: true };
    /// Both channels.
    pub const AB: Self = Self { a: true, b: true };

    /// Returns whether the set contains a channel.
    #[must_use]
    pub const fn contains(self, channel: Channel) -> bool {
        match channel {
            Channel::A => self.a,
            Channel::B => self.b,
        }
    }

    /// Returns the number of channels in the set.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.a as u8 + self.b as u8
    }
}

/// Intensity control capabilities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntensityCapabilities {
    /// Whether absolute intensity can be set directly.
    pub absolute: bool,
    /// Whether intensity can be adjusted relative to the current value.
    pub relative: bool,
    /// Whether temporary intensity is supported.
    pub temporary: bool,
    /// Maximum supported intensity.
    pub max: Intensity,
}

/// Waveform capabilities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaveformCapabilities {
    /// Whether V2 pulse frames are supported.
    pub v2_frames: bool,
    /// Whether V3 pulse frames are supported.
    pub v3_frames: bool,
    /// Maximum playback duration, when limited.
    pub max_duration: Option<Duration>,
}

/// A summary of what a device can do.
///
/// Capabilities are advisory. Operations must still validate the concrete values
/// and return an explicit unsupported/invalid error when necessary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceCapabilities {
    /// Channels exposed by the device.
    pub channels: ChannelSet,
    /// Whether battery reporting is available.
    pub battery: bool,
    /// Intensity capabilities.
    pub intensity: IntensityCapabilities,
    /// Waveform capabilities.
    pub waveform: WaveformCapabilities,
    /// Whether device settings can be read or written.
    pub settings: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_set() {
        assert_eq!(ChannelSet::AB.count(), 2);
        assert!(ChannelSet::AB.contains(Channel::A));
        assert!(ChannelSet::AB.contains(Channel::B));
        assert!(!ChannelSet::A.contains(Channel::B));
        assert_eq!(ChannelSet::default(), ChannelSet::NONE);
    }
}
