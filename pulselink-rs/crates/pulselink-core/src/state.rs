//! Device connection state, observed values, and state snapshots.

use std::fmt;

use crate::channel::{Channel, ChannelMap};
use crate::error::CoreError;
use crate::intensity::Intensity;

/// The lifecycle state of a device session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ConnectionState {
    /// No link is established.
    Disconnected,
    /// A link is being established.
    Connecting,
    /// The link is up but initialization is not complete.
    Initializing,
    /// The session is initialized and ready for control.
    Ready,
    /// The link is unreliable but the session is not closed yet.
    Degraded,
    /// The session is trying to re-establish the link.
    Reconnecting,
    /// The session is shutting down.
    Closing,
}

/// A value that may not have been observed yet.
///
/// `Observed::Unknown` is not the same as `Observed::Known(0)`. A device that has
/// just connected has not necessarily reported its intensity, battery, or settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Observed<T> {
    /// The value has not been observed.
    Unknown,
    /// The value is known.
    Known(T),
}

impl<T> Observed<T> {
    /// Creates an unknown value.
    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    /// Creates a known value.
    #[must_use]
    pub const fn known(value: T) -> Self {
        Self::Known(value)
    }

    /// Returns whether the value is known.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    /// Returns whether the value is unknown.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns the known value, if any.
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Unknown => None,
            Self::Known(value) => Some(value),
        }
    }

    /// Borrows the observed value.
    #[must_use]
    pub const fn as_ref(&self) -> Observed<&T> {
        match self {
            Self::Unknown => Observed::Unknown,
            Self::Known(value) => Observed::Known(value),
        }
    }
}

impl<T: Copy> Observed<T> {
    /// Returns a copy of the known value, if any.
    #[must_use]
    pub const fn copied(self) -> Option<T> {
        match self {
            Self::Unknown => None,
            Self::Known(value) => Some(value),
        }
    }
}

#[allow(
    clippy::derivable_impls,
    reason = "deriving Default would add an unnecessary T: Default bound"
)]
impl<T> Default for Observed<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A battery percentage in the range `0..=100`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
pub struct BatteryLevel(u8);

impl BatteryLevel {
    /// The minimum battery level.
    pub const MIN: Self = Self(0);
    /// The maximum battery level.
    pub const MAX: Self = Self(100);

    /// Creates a battery level.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidBatteryLevel`] when `value > 100`.
    pub const fn new(value: u8) -> Result<Self, CoreError> {
        if value <= Self::MAX.0 {
            Ok(Self(value))
        } else {
            Err(CoreError::InvalidBatteryLevel(value))
        }
    }

    /// Returns the raw battery percentage.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for BatteryLevel {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<BatteryLevel> for u8 {
    fn from(value: BatteryLevel) -> Self {
        value.0
    }
}

impl fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// Observed battery information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BatteryState {
    /// Current battery percentage.
    pub level: BatteryLevel,
}

impl BatteryState {
    /// Creates battery state from a level.
    #[must_use]
    pub const fn new(level: BatteryLevel) -> Self {
        Self { level }
    }
}

/// Observed state for one channel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelState {
    /// Last observed channel intensity.
    pub intensity: Observed<Intensity>,
    /// Last observed intensity soft limit.
    pub soft_limit: Observed<Intensity>,
}

/// A snapshot of a device's observed state.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceState {
    revision: u64,
    /// Current lifecycle state.
    pub connection: ConnectionState,
    /// Last observed battery state.
    pub battery: Observed<BatteryState>,
    /// Per-channel observed state.
    pub channels: ChannelMap<ChannelState>,
}

impl DeviceState {
    /// Creates an empty state snapshot with the given connection state.
    #[must_use]
    pub fn new(connection: ConnectionState) -> Self {
        Self {
            revision: 0,
            connection,
            battery: Observed::Unknown,
            channels: ChannelMap::default(),
        }
    }

    /// Returns the monotonically increasing snapshot revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Marks the snapshot as updated and returns the new revision.
    pub fn touch(&mut self) -> u64 {
        self.revision = self.revision.saturating_add(1);
        self.revision
    }

    /// Returns the observed state for a channel.
    #[must_use]
    pub const fn channel(&self, channel: Channel) -> &ChannelState {
        self.channels.get(channel)
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new(ConnectionState::Disconnected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_distinguishes_unknown_from_zero() {
        let unknown: Observed<u8> = Observed::Unknown;
        let zero = Observed::Known(0_u8);
        assert!(unknown.is_unknown());
        assert!(zero.is_known());
        assert_eq!(unknown.copied(), None);
        assert_eq!(zero.copied(), Some(0));
        assert_eq!(Observed::<u8>::default(), Observed::Unknown);
    }

    #[test]
    fn battery_level_bounds() {
        assert_eq!(BatteryLevel::new(0).map(BatteryLevel::get), Ok(0));
        assert_eq!(BatteryLevel::new(100).map(BatteryLevel::get), Ok(100));
        assert_eq!(
            BatteryLevel::new(101),
            Err(CoreError::InvalidBatteryLevel(101))
        );
        assert_eq!(BatteryLevel::MAX.to_string(), "100%");
    }

    #[test]
    fn device_state_revision_and_channels() {
        let mut state = DeviceState::new(ConnectionState::Connecting);
        assert_eq!(state.revision(), 0);
        assert_eq!(state.touch(), 1);
        assert_eq!(state.touch(), 2);
        assert_eq!(state.revision(), 2);
        assert!(state.channel(Channel::A).intensity.is_unknown());
        state.channels.a.intensity = Observed::Known(Intensity::new(10).expect("valid"));
        assert!(state.channel(Channel::A).intensity.is_known());
        assert!(state.channel(Channel::B).intensity.is_unknown());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn device_state_serde_roundtrip() {
        let mut state = DeviceState::new(ConnectionState::Ready);
        state.battery = Observed::Known(BatteryState::new(BatteryLevel::new(80).expect("valid")));
        state.channels.a.intensity = Observed::Known(Intensity::new(12).expect("valid"));
        state.touch();

        let json = serde_json::to_string(&state).expect("serialize");
        let roundtrip: DeviceState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, state);
    }
}
