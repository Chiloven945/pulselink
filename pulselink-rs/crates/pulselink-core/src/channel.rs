//! Channels and channel-indexed values.

use std::fmt;
use std::ops::{Index, IndexMut};

use crate::error::CoreError;

/// A device output channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Channel {
    /// Channel A.
    A,
    /// Channel B.
    B,
}

impl Channel {
    /// Both channels in A, B order.
    pub const ALL: [Self; 2] = [Self::A, Self::B];

    /// Returns the channel index (`A` = 0, `B` = 1).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    /// Returns the wire discriminator (`A` = 0, `B` = 1).
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    /// Converts a wire discriminator into a channel.
    pub const fn from_u8(value: u8) -> Result<Self, CoreError> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            other => Err(CoreError::InvalidChannel(other)),
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => f.write_str("A"),
            Self::B => f.write_str("B"),
        }
    }
}

/// A value that exists independently for both channels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelMap<T> {
    /// Value for channel A.
    pub a: T,
    /// Value for channel B.
    pub b: T,
}

impl<T> ChannelMap<T> {
    /// Creates a channel map from the two channel values.
    #[must_use]
    pub const fn new(a: T, b: T) -> Self {
        Self { a, b }
    }

    /// Returns the value for the requested channel.
    #[must_use]
    pub const fn get(&self, channel: Channel) -> &T {
        match channel {
            Channel::A => &self.a,
            Channel::B => &self.b,
        }
    }

    /// Returns a mutable reference to the value for the requested channel.
    pub fn get_mut(&mut self, channel: Channel) -> &mut T {
        match channel {
            Channel::A => &mut self.a,
            Channel::B => &mut self.b,
        }
    }

    /// Iterates over both channel values.
    pub fn iter(&self) -> impl Iterator<Item = (Channel, &T)> {
        [(Channel::A, &self.a), (Channel::B, &self.b)].into_iter()
    }

    /// Applies a function to both channel values.
    #[must_use]
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ChannelMap<U> {
        ChannelMap {
            a: f(self.a),
            b: f(self.b),
        }
    }

    /// Borrows both channel values.
    #[must_use]
    pub const fn as_ref(&self) -> ChannelMap<&T> {
        ChannelMap {
            a: &self.a,
            b: &self.b,
        }
    }
}

impl<T> Index<Channel> for ChannelMap<T> {
    type Output = T;

    fn index(&self, channel: Channel) -> &Self::Output {
        self.get(channel)
    }
}

impl<T> IndexMut<Channel> for ChannelMap<T> {
    fn index_mut(&mut self, channel: Channel) -> &mut Self::Output {
        self.get_mut(channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_discriminators() {
        assert_eq!(Channel::A.index(), 0);
        assert_eq!(Channel::B.index(), 1);
        assert_eq!(Channel::A.as_u8(), 0);
        assert_eq!(Channel::B.as_u8(), 1);
        assert_eq!(Channel::from_u8(0), Ok(Channel::A));
        assert_eq!(Channel::from_u8(1), Ok(Channel::B));
        assert_eq!(Channel::from_u8(2), Err(CoreError::InvalidChannel(2)));
    }

    #[test]
    fn channel_map_access() {
        let mut map = ChannelMap::new(10, 20);
        assert_eq!(*map.get(Channel::A), 10);
        assert_eq!(*map.get(Channel::B), 20);
        *map.get_mut(Channel::A) = 30;
        assert_eq!(map[Channel::A], 30);
        assert_eq!(map[Channel::B], 20);
        assert_eq!(map.map(|value| value * 2), ChannelMap::new(60, 40));
        assert_eq!(map.as_ref().a, &30);
    }

    #[test]
    fn channel_map_iteration() {
        let map = ChannelMap::new("a", "b");
        let values: Vec<_> = map.iter().collect();
        assert_eq!(values, vec![(Channel::A, &"a"), (Channel::B, &"b")]);
    }
}
