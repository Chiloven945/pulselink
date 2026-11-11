//! Coyote V3 `BF` settings command encoding.

use pulselink_core::{ChannelMap, Intensity};

/// A Coyote V3 `BF` settings command.
///
/// BF takes effect immediately and has no response. It should be re-sent after
/// every reconnect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BfCommand {
    /// Per-channel intensity soft limits.
    pub soft_limit: ChannelMap<Intensity>,
    /// Per-channel frequency balance parameters.
    pub frequency_balance: ChannelMap<u8>,
    /// Per-channel intensity balance parameters.
    pub intensity_balance: ChannelMap<u8>,
}

impl BfCommand {
    /// Encodes the command into 7 bytes.
    #[must_use]
    pub fn encode(&self) -> [u8; 7] {
        [
            0xBF,
            self.soft_limit.a.get(),
            self.soft_limit.b.get(),
            self.frequency_balance.a,
            self.frequency_balance.b,
            self.intensity_balance.a,
            self.intensity_balance.b,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let command = BfCommand {
            soft_limit: ChannelMap::new(
                Intensity::new(50).expect("valid intensity"),
                Intensity::new(60).expect("valid intensity"),
            ),
            frequency_balance: ChannelMap::new(128, 129),
            intensity_balance: ChannelMap::new(130, 131),
        };

        assert_eq!(command.encode(), [0xBF, 50, 60, 128, 129, 130, 131]);
    }

    #[test]
    fn boundary_values() {
        let command = BfCommand {
            soft_limit: ChannelMap::new(Intensity::MIN, Intensity::MAX),
            frequency_balance: ChannelMap::new(0, 255),
            intensity_balance: ChannelMap::new(0, 255),
        };

        assert_eq!(command.encode(), [0xBF, 0, 200, 0, 255, 0, 255]);
    }
}
