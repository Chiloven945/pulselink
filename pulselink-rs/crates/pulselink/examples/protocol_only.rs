//! Use the sans-io protocol without any BLE or WebSocket dependency.

use pulselink::Intensity;
use pulselink::protocol::coyote::v3::{
    B0Command, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = B0Command {
        sequence: Sequence::new(1)?,
        intensity: StereoIntensityOp {
            a: IntensityOp::Set(Intensity::MIN),
            b: IntensityOp::Keep,
        },
        waveform: StereoWaveform {
            a: ChannelWaveform::Silent,
            b: ChannelWaveform::Silent,
        },
    };
    println!("{:02X?}", command.encode());
    Ok(())
}
