//! Encode a Coyote V3 B0 command without any I/O.

use std::fmt::Write as _;

use pulselink_core::Intensity;
use pulselink_pattern::{PulseIntensity, PulseInterval, PulseSample};
use pulselink_protocol::coyote::v3::{
    B0Command, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = |interval, intensity| -> Result<PulseSample, Box<dyn std::error::Error>> {
        Ok(PulseSample::new(
            PulseInterval::new(interval)?,
            PulseIntensity::new(intensity)?,
        ))
    };

    let command = B0Command {
        sequence: Sequence::new(1)?,
        intensity: StereoIntensityOp {
            a: IntensityOp::Set(Intensity::new(10)?),
            b: IntensityOp::Keep,
        },
        waveform: StereoWaveform {
            a: ChannelWaveform::Active([
                sample(100, 20)?,
                sample(100, 30)?,
                sample(100, 20)?,
                sample(100, 10)?,
            ]),
            b: ChannelWaveform::Silent,
        },
    };

    let bytes = command.encode();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex, "{byte:02X}")?;
    }
    println!("{hex}");

    Ok(())
}
