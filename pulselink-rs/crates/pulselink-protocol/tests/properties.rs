//! Property tests for protocol codecs.

#![cfg(any(feature = "coyote-v3", feature = "socket-v4"))]

#[cfg(feature = "coyote-v3")]
mod coyote_v3 {
    use proptest::prelude::*;
    use pulselink_core::Intensity;
    use pulselink_pattern::{PulseIntensity, PulseInterval, PulseSample};
    use pulselink_protocol::coyote::v3::{
        B0Command, B1Message, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp,
        StereoWaveform, decode_wire_frequency,
    };

    proptest! {
        #[test]
        fn b0_encode_has_fixed_shape(
            sequence in 0_u8..=15,
            a in 0_u8..=200,
            b in 0_u8..=200,
        ) {
            let command = B0Command {
                sequence: Sequence::new(sequence).expect("valid sequence"),
                intensity: StereoIntensityOp {
                    a: IntensityOp::Set(Intensity::new(a).expect("valid intensity")),
                    b: IntensityOp::Set(Intensity::new(b).expect("valid intensity")),
                },
                waveform: StereoWaveform {
                    a: ChannelWaveform::Silent,
                    b: ChannelWaveform::Silent,
                },
            };

            let bytes = command.encode();
            prop_assert_eq!(bytes.len(), 20);
            prop_assert_eq!(bytes[0], 0xB0);
            prop_assert_eq!(bytes[1] >> 4, sequence);
            prop_assert_eq!(bytes[1] & 0b11, 0b11);
            prop_assert_eq!(bytes[2], a);
            prop_assert_eq!(bytes[3], b);
        }

        #[test]
        fn b0_active_waveform_is_encodable(
            interval in 10_u16..=1000,
            intensity in 0_u8..=100,
        ) {
            let sample = PulseSample::new(
                PulseInterval::new(interval).expect("valid interval"),
                PulseIntensity::new(intensity).expect("valid intensity"),
            );
            let command = B0Command {
                sequence: Sequence::ZERO,
                intensity: StereoIntensityOp {
                    a: IntensityOp::Keep,
                    b: IntensityOp::Keep,
                },
                waveform: StereoWaveform {
                    a: ChannelWaveform::Active([sample; 4]),
                    b: ChannelWaveform::Silent,
                },
            };

            let bytes = command.encode();
            prop_assert!(bytes[4] >= 10);
            prop_assert!(bytes[4] <= 240);
            prop_assert_eq!(bytes[8], intensity);
        }

        #[test]
        fn b1_decode_never_panics(bytes in proptest::array::uniform4(any::<u8>())) {
            let _ = B1Message::decode(&bytes);
        }

        #[test]
        fn every_wire_frequency_has_a_representative(wire in 10_u8..=240) {
            prop_assert!(decode_wire_frequency(wire).is_ok());
        }
    }
}

#[cfg(feature = "socket-v4")]
mod socket_v4 {
    use proptest::prelude::*;
    use pulselink_protocol::socket::v4::{AppMessage, RelayFrame};

    proptest! {
        #[test]
        fn relay_frame_json_never_panics(input in any::<String>()) {
            let _ = serde_json::from_str::<RelayFrame>(&input);
        }

        #[test]
        fn app_message_json_never_panics(input in any::<String>()) {
            let _ = serde_json::from_str::<AppMessage>(&input);
        }
    }
}
