//! Coyote V3 `B0` control command encoding.

use pulselink_core::Intensity;
use pulselink_pattern::PulseSample;

use crate::coyote::v3::frequency;
use crate::error::ProtocolError;

/// A 4-bit B0 sequence number.
///
/// `0` means the device does not need to send a B1 response. Intensity changes
/// should use `1..=15` and wait for the matching B1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sequence(u8);

impl Sequence {
    /// Sequence zero, meaning no B1 response is required.
    pub const ZERO: Self = Self(0);
    /// The largest valid sequence number.
    pub const MAX: u8 = 15;

    /// Creates a sequence number in `0..=15`.
    pub const fn new(value: u8) -> Result<Self, ProtocolError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(ProtocolError::InvalidSequence(value))
        }
    }

    /// Returns the raw sequence number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// The intensity operation for one channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntensityOp {
    /// Keep the current intensity; the setting byte is ignored.
    Keep,
    /// Increase by the given value.
    Increase(Intensity),
    /// Decrease by the given value.
    Decrease(Intensity),
    /// Set an absolute intensity.
    Set(Intensity),
}

impl IntensityOp {
    fn encode(self) -> (u8, u8) {
        match self {
            Self::Keep => (0b00, 0),
            Self::Increase(value) => (0b01, value.get()),
            Self::Decrease(value) => (0b10, value.get()),
            Self::Set(value) => (0b11, value.get()),
        }
    }
}

/// Intensity operations for both channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StereoIntensityOp {
    /// Channel A operation.
    pub a: IntensityOp,
    /// Channel B operation.
    pub b: IntensityOp,
}

/// Waveform data for one channel in a 100 ms B0 frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelWaveform {
    /// Four 25 ms pulse samples.
    Active([PulseSample; 4]),
    /// Explicit silence for the 100 ms window.
    Silent,
}

impl ChannelWaveform {
    fn encode_into(self, output: &mut [u8; 8]) {
        match self {
            Self::Active(samples) => {
                for (index, sample) in samples.iter().enumerate() {
                    output[index] = frequency::encode_interval(sample.interval);
                    output[4 + index] = sample.intensity.get();
                }
            }
            Self::Silent => {
                output.fill(0);
                // An amplitude above 100 makes the device discard this channel's
                // four pairs. The official silence convention uses 101.
                output[7] = 101;
            }
        }
    }
}

/// Waveform data for both channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StereoWaveform {
    /// Channel A waveform.
    pub a: ChannelWaveform,
    /// Channel B waveform.
    pub b: ChannelWaveform,
}

/// A Coyote V3 `B0` control command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct B0Command {
    /// Request sequence number.
    pub sequence: Sequence,
    /// Per-channel intensity operations.
    pub intensity: StereoIntensityOp,
    /// Per-channel waveform data.
    pub waveform: StereoWaveform,
}

impl B0Command {
    /// Encodes the command into 20 bytes.
    #[must_use]
    pub fn encode(&self) -> [u8; 20] {
        let (mode_a, intensity_a) = self.intensity.a.encode();
        let (mode_b, intensity_b) = self.intensity.b.encode();

        let mut output = [0_u8; 20];
        output[0] = 0xB0;
        output[1] = (self.sequence.get() << 4) | (mode_a << 2) | mode_b;
        output[2] = intensity_a;
        output[3] = intensity_b;

        let mut a = [0_u8; 8];
        self.waveform.a.encode_into(&mut a);
        output[4..12].copy_from_slice(&a);

        let mut b = [0_u8; 8];
        self.waveform.b.encode_into(&mut b);
        output[12..20].copy_from_slice(&b);

        output
    }
}

#[cfg(test)]
mod tests {
    use pulselink_pattern::{PulseIntensity, PulseInterval};

    use super::*;

    fn sample(interval: u16, intensity: u8) -> PulseSample {
        PulseSample::new(
            PulseInterval::new(interval).expect("valid interval"),
            PulseIntensity::new(intensity).expect("valid intensity"),
        )
    }

    fn intensity(value: u8) -> Intensity {
        Intensity::new(value).expect("valid intensity")
    }

    fn hex(bytes: [u8; 20]) -> String {
        use std::fmt::Write as _;

        let mut output = String::with_capacity(40);
        for byte in bytes {
            write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
        }
        output
    }

    #[test]
    fn sequence_bounds() {
        assert_eq!(Sequence::new(0), Ok(Sequence::ZERO));
        assert_eq!(Sequence::new(15).map(Sequence::get), Ok(15));
        assert_eq!(Sequence::new(16), Err(ProtocolError::InvalidSequence(16)));
    }

    #[test]
    fn official_no1_first_frame() {
        let command = B0Command {
            sequence: Sequence::ZERO,
            intensity: StereoIntensityOp {
                a: IntensityOp::Keep,
                b: IntensityOp::Keep,
            },
            waveform: StereoWaveform {
                a: ChannelWaveform::Active([
                    sample(10, 0),
                    sample(10, 10),
                    sample(10, 20),
                    sample(10, 30),
                ]),
                b: ChannelWaveform::Silent,
            },
        };

        assert_eq!(
            hex(command.encode()),
            "B00000000A0A0A0A000A141E0000000000000065"
        );
    }

    #[test]
    fn official_no2_first_frame() {
        let command = B0Command {
            sequence: Sequence::ZERO,
            intensity: StereoIntensityOp {
                a: IntensityOp::Increase(intensity(5)),
                b: IntensityOp::Keep,
            },
            waveform: StereoWaveform {
                a: ChannelWaveform::Active([
                    sample(10, 0),
                    sample(10, 10),
                    sample(10, 20),
                    sample(10, 30),
                ]),
                b: ChannelWaveform::Silent,
            },
        };

        assert_eq!(
            hex(command.encode()),
            "B00405000A0A0A0A000A141E0000000000000065"
        );
    }

    #[test]
    fn official_no2_sequence_one_frame() {
        let command = B0Command {
            sequence: Sequence::new(1).expect("valid sequence"),
            intensity: StereoIntensityOp {
                a: IntensityOp::Increase(intensity(10)),
                b: IntensityOp::Keep,
            },
            waveform: StereoWaveform {
                a: ChannelWaveform::Active([
                    sample(40, 100),
                    sample(60, 90),
                    sample(80, 90),
                    sample(100, 90),
                ]),
                b: ChannelWaveform::Silent,
            },
        };

        assert_eq!(
            hex(command.encode()),
            "B0140A00283C5064645A5A5A0000000000000065"
        );
    }

    #[test]
    fn mode_bits_and_intensity_bytes() {
        let command = B0Command {
            sequence: Sequence::new(15).expect("valid sequence"),
            intensity: StereoIntensityOp {
                a: IntensityOp::Set(intensity(200)),
                b: IntensityOp::Decrease(intensity(7)),
            },
            waveform: StereoWaveform {
                a: ChannelWaveform::Silent,
                b: ChannelWaveform::Silent,
            },
        };

        let bytes = command.encode();
        assert_eq!(bytes[1], (15 << 4) | (0b11 << 2) | 0b10);
        assert_eq!(bytes[2], 200);
        assert_eq!(bytes[3], 7);
        assert_eq!(&bytes[4..8], &[0, 0, 0, 0]);
        assert_eq!(&bytes[8..12], &[0, 0, 0, 101]);
        assert_eq!(&bytes[12..16], &[0, 0, 0, 0]);
        assert_eq!(&bytes[16..20], &[0, 0, 0, 101]);
    }

    #[test]
    fn active_waveform_uses_wire_frequency() {
        let command = B0Command {
            sequence: Sequence::ZERO,
            intensity: StereoIntensityOp {
                a: IntensityOp::Keep,
                b: IntensityOp::Keep,
            },
            waveform: StereoWaveform {
                a: ChannelWaveform::Active([
                    sample(10, 0),
                    sample(100, 100),
                    sample(600, 50),
                    sample(1000, 25),
                ]),
                b: ChannelWaveform::Silent,
            },
        };

        let bytes = command.encode();
        assert_eq!(&bytes[4..8], &[10, 100, 200, 240]);
        assert_eq!(&bytes[8..12], &[0, 100, 50, 25]);
    }
}
