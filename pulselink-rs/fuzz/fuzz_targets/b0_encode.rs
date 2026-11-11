#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_core::{ChannelMap, Intensity};
use pulselink_pattern::{PulseIntensity, PulseInterval, PulseSample};
use pulselink_protocol::coyote::v3::{
    B0Command, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }
    let sample = |raw: u8| {
        let interval = 10 + u16::from(raw) % 991;
        PulseSample::new(
            PulseInterval::new(interval).expect("valid interval"),
            PulseIntensity::new(raw % 101).expect("valid intensity"),
        )
    };
    let command = B0Command {
        sequence: Sequence::new(data[0] & 0x0F).expect("valid sequence"),
        intensity: StereoIntensityOp {
            a: IntensityOp::Set(Intensity::new(data[1] % 201).expect("valid intensity")),
            b: IntensityOp::Keep,
        },
        waveform: StereoWaveform {
            a: ChannelWaveform::Active([
                sample(data[2]),
                sample(data[3]),
                sample(data[4]),
                sample(data[5]),
            ]),
            b: ChannelWaveform::Silent,
        },
    };
    let bytes = command.encode();
    assert_eq!(bytes.len(), 20);

    let bf = pulselink_protocol::coyote::v3::BfCommand {
        soft_limit: ChannelMap::new(
            Intensity::new(data[6] % 201).expect("valid intensity"),
            Intensity::new(data[7] % 201).expect("valid intensity"),
        ),
        frequency_balance: ChannelMap::new(data[8], data[0]),
        intensity_balance: ChannelMap::new(data[1], data[2]),
    };
    assert_eq!(bf.encode().len(), 7);
});
