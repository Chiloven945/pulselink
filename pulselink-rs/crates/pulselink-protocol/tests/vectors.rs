//! Cross-language test vector runner.
//!
//! Canonical vectors live in `spec/test-vectors/`. The copies under
//! `tests/vectors/` are synchronized by `scripts/sync-vectors.sh`.

#![cfg(any(feature = "coyote-v3", feature = "socket-v4"))]

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct VectorFile {
    version: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    kind: String,
    input: Value,
    expect: Value,
}

fn load(json: &str) -> VectorFile {
    serde_json::from_str(json).expect("valid vector file")
}

#[cfg(feature = "coyote-v3")]
fn hex_decode(input: &str) -> Vec<u8> {
    assert_eq!(input.len() % 2, 0, "hex input must contain full bytes");
    input
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = char::from(pair[0])
                .to_digit(16)
                .expect("valid hex high nibble");
            let low = char::from(pair[1])
                .to_digit(16)
                .expect("valid hex low nibble");
            u8::try_from((high << 4) | low).expect("valid byte")
        })
        .collect()
}

#[cfg(feature = "coyote-v3")]
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(feature = "coyote-v3")]
fn expect_error(case: &Case) -> bool {
    case.expect
        .get("error")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[cfg(feature = "coyote-v3")]
fn value_u8(value: &Value, key: &str) -> u8 {
    u8::try_from(value[key].as_u64().expect("u8 value")).expect("u8 range")
}

#[cfg(feature = "coyote-v3")]
fn value_u16(value: &Value, key: &str) -> u16 {
    u16::try_from(value[key].as_u64().expect("u16 value")).expect("u16 range")
}

#[cfg(feature = "coyote-v3")]
#[test]
fn coyote_v3_vectors() {
    let files = [
        include_str!("vectors/coyote-v3/b0.json"),
        include_str!("vectors/coyote-v3/bf.json"),
        include_str!("vectors/coyote-v3/b1.json"),
        include_str!("vectors/coyote-v3/battery.json"),
        include_str!("vectors/coyote-v3/frequency.json"),
        include_str!("vectors/coyote-v3/invalid.json"),
    ];

    for json in files {
        let file = load(json);
        assert_eq!(file.version, 1);
        for case in file.cases {
            run_coyote_v3_case(&case);
        }
    }
}

#[cfg(feature = "coyote-v3")]
fn run_coyote_v3_case(case: &Case) {
    match case.kind.as_str() {
        "b0.encode" => {
            let input: B0Input =
                serde_json::from_value(case.input.clone()).expect("valid B0 input");
            let command = input.into_command();
            let expected = case.expect["hex"].as_str().expect("expected hex");
            assert_eq!(hex_encode(&command.encode()), expected, "case {}", case.id);
        }
        "bf.encode" => {
            let input: BfInput =
                serde_json::from_value(case.input.clone()).expect("valid BF input");
            let expected = case.expect["hex"].as_str().expect("expected hex");
            assert_eq!(
                hex_encode(&input.into_command().encode()),
                expected,
                "case {}",
                case.id
            );
        }
        "b1.decode" => {
            let bytes = hex_decode(case.input["hex"].as_str().expect("B1 hex"));
            match B1Message::decode(&bytes) {
                Ok(message) => {
                    assert!(
                        !expect_error(case),
                        "case {} unexpectedly succeeded",
                        case.id
                    );
                    assert_eq!(
                        u64::from(message.sequence.get()),
                        case.expect["sequence"].as_u64().expect("sequence"),
                        "case {}",
                        case.id
                    );
                    assert_eq!(
                        u64::from(message.intensity.a.get()),
                        case.expect["a"].as_u64().expect("channel A"),
                        "case {}",
                        case.id
                    );
                    assert_eq!(
                        u64::from(message.intensity.b.get()),
                        case.expect["b"].as_u64().expect("channel B"),
                        "case {}",
                        case.id
                    );
                }
                Err(error) => assert!(expect_error(case), "case {} failed: {error}", case.id),
            }
        }
        "battery.decode" => {
            let bytes = hex_decode(case.input["hex"].as_str().expect("battery hex"));
            match decode_battery(&bytes) {
                Ok(level) => {
                    assert!(
                        !expect_error(case),
                        "case {} unexpectedly succeeded",
                        case.id
                    );
                    assert_eq!(
                        u64::from(level.get()),
                        case.expect["level"].as_u64().expect("level"),
                        "case {}",
                        case.id
                    );
                }
                Err(error) => assert!(expect_error(case), "case {} failed: {error}", case.id),
            }
        }
        "frequency.encode" => {
            let interval =
                PulseInterval::new(value_u16(&case.input, "interval_ms")).expect("valid interval");
            assert_eq!(
                u64::from(encode_interval(interval)),
                case.expect["wire"].as_u64().expect("wire"),
                "case {}",
                case.id
            );
        }
        "frequency.decode" => {
            let wire = value_u8(&case.input, "wire");
            match decode_wire_frequency(wire) {
                Ok(interval) => {
                    assert!(
                        !expect_error(case),
                        "case {} unexpectedly succeeded",
                        case.id
                    );
                    assert_eq!(
                        u64::from(interval.get()),
                        case.expect["interval_ms"].as_u64().expect("interval"),
                        "case {}",
                        case.id
                    );
                }
                Err(error) => assert!(expect_error(case), "case {} failed: {error}", case.id),
            }
        }
        other => panic!("unknown Coyote V3 vector kind: {other}"),
    }
}

#[cfg(feature = "coyote-v3")]
use pulselink_core::{ChannelMap, Intensity};
#[cfg(feature = "coyote-v3")]
use pulselink_pattern::{PulseIntensity, PulseInterval, PulseSample};
#[cfg(feature = "coyote-v3")]
use pulselink_protocol::coyote::v3::{
    B0Command, B1Message, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp,
    StereoWaveform, decode_battery, decode_wire_frequency, encode_interval,
};

#[cfg(feature = "coyote-v3")]
#[derive(Deserialize)]
struct B0Input {
    sequence: u8,
    intensity_a: IntensityInput,
    intensity_b: IntensityInput,
    waveform_a: WaveformInput,
    waveform_b: WaveformInput,
}

#[cfg(feature = "coyote-v3")]
#[derive(Deserialize)]
#[serde(tag = "op", content = "value", rename_all = "snake_case")]
enum IntensityInput {
    Keep,
    Increase(u8),
    Decrease(u8),
    Set(u8),
}

#[cfg(feature = "coyote-v3")]
impl IntensityInput {
    fn into_op(self) -> IntensityOp {
        match self {
            Self::Keep => IntensityOp::Keep,
            Self::Increase(value) => {
                IntensityOp::Increase(Intensity::new(value).expect("valid intensity"))
            }
            Self::Decrease(value) => {
                IntensityOp::Decrease(Intensity::new(value).expect("valid intensity"))
            }
            Self::Set(value) => IntensityOp::Set(Intensity::new(value).expect("valid intensity")),
        }
    }
}

#[cfg(feature = "coyote-v3")]
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WaveformInput {
    Silent,
    Active {
        frequencies: [u16; 4],
        intensities: [u8; 4],
    },
}

#[cfg(feature = "coyote-v3")]
impl WaveformInput {
    fn into_waveform(self) -> ChannelWaveform {
        match self {
            Self::Silent => ChannelWaveform::Silent,
            Self::Active {
                frequencies,
                intensities,
            } => ChannelWaveform::Active(std::array::from_fn(|index| {
                PulseSample::new(
                    PulseInterval::new(frequencies[index]).expect("valid interval"),
                    PulseIntensity::new(intensities[index]).expect("valid intensity"),
                )
            })),
        }
    }
}

#[cfg(feature = "coyote-v3")]
impl B0Input {
    fn into_command(self) -> B0Command {
        B0Command {
            sequence: Sequence::new(self.sequence).expect("valid sequence"),
            intensity: StereoIntensityOp {
                a: self.intensity_a.into_op(),
                b: self.intensity_b.into_op(),
            },
            waveform: StereoWaveform {
                a: self.waveform_a.into_waveform(),
                b: self.waveform_b.into_waveform(),
            },
        }
    }
}

#[cfg(feature = "coyote-v3")]
#[derive(Deserialize)]
struct BfInput {
    soft_limit_a: u8,
    soft_limit_b: u8,
    frequency_balance_a: u8,
    frequency_balance_b: u8,
    intensity_balance_a: u8,
    intensity_balance_b: u8,
}

#[cfg(feature = "coyote-v3")]
impl BfInput {
    fn into_command(self) -> pulselink_protocol::coyote::v3::BfCommand {
        pulselink_protocol::coyote::v3::BfCommand {
            soft_limit: ChannelMap::new(
                Intensity::new(self.soft_limit_a).expect("valid intensity"),
                Intensity::new(self.soft_limit_b).expect("valid intensity"),
            ),
            frequency_balance: ChannelMap::new(self.frequency_balance_a, self.frequency_balance_b),
            intensity_balance: ChannelMap::new(self.intensity_balance_a, self.intensity_balance_b),
        }
    }
}

#[cfg(feature = "socket-v4")]
#[test]
fn websocket_v4_vectors() {
    let files = [
        include_str!("vectors/websocket-v4/frames.json"),
        include_str!("vectors/websocket-v4/rpc.json"),
        include_str!("vectors/websocket-v4/device-op.json"),
        include_str!("vectors/websocket-v4/events.json"),
        include_str!("vectors/websocket-v4/invalid.json"),
    ];

    for json in files {
        let file = load(json);
        assert_eq!(file.version, 1);
        for case in file.cases {
            run_websocket_v4_case(&case);
        }
    }
}

#[cfg(feature = "socket-v4")]
use pulselink_protocol::socket::v4::{
    ActionType, AppEvent, AppMessage, ClearOperate, DeviceOperate, DeviceOperateResult, RelayFrame,
};

#[cfg(feature = "socket-v4")]
fn relay_variant(frame: &RelayFrame) -> &'static str {
    match frame {
        RelayFrame::Hello { .. } => "hello",
        RelayFrame::ClientAttached { .. } => "client_attached",
        RelayFrame::ClientDisconnected { .. } => "client_disconnected",
        RelayFrame::ControllerAttached { .. } => "controller_attached",
        RelayFrame::ControllerDisconnected { .. } => "controller_disconnected",
        RelayFrame::Heartbeat => "heartbeat",
        RelayFrame::Ping => "ping",
        RelayFrame::Pong { .. } => "pong",
        RelayFrame::Message { .. } => "message",
        RelayFrame::IdleTimeout => "idle_timeout",
        RelayFrame::Error { .. } => "error",
        RelayFrame::Unknown => "unknown",
    }
}

#[cfg(feature = "socket-v4")]
fn app_event_variant(event: &AppEvent) -> &'static str {
    match event {
        AppEvent::DevicesSnapshot { .. } => "devices_snapshot",
        AppEvent::DevicesPatch { .. } => "devices_patch",
        AppEvent::SlotsPatch { .. } => "slots_patch",
        AppEvent::CustomAction { .. } => "custom_action",
        AppEvent::Unknown => "unknown",
    }
}

#[cfg(feature = "socket-v4")]
fn action_name(action: ActionType) -> &'static str {
    match action {
        ActionType::AppendPulseData => "append_pulse_data",
        ActionType::AddIntensity => "add_intensity",
        ActionType::SetTempIntensity => "set_temp_intensity",
        ActionType::SetIntensity => "set_intensity",
    }
}

#[cfg(feature = "socket-v4")]
#[allow(clippy::too_many_lines, reason = "test vector dispatcher")]
fn run_websocket_v4_case(case: &Case) {
    match case.kind.as_str() {
        "relay_frame" => {
            let frame: RelayFrame =
                serde_json::from_value(case.input.clone()).expect("valid relay frame");
            assert_eq!(
                relay_variant(&frame),
                case.expect["variant"].as_str().expect("variant"),
                "case {}",
                case.id
            );
        }
        "relay_frame_invalid" => {
            assert!(
                serde_json::from_value::<RelayFrame>(case.input.clone()).is_err(),
                "case {} should fail",
                case.id
            );
        }
        "app_message" => {
            let message: AppMessage =
                serde_json::from_value(case.input.clone()).expect("valid app message");
            let variant = match &message {
                AppMessage::Req(_) => "req",
                AppMessage::Resp(_) => "resp",
                AppMessage::Ev(_) => "ev",
            };
            assert_eq!(
                variant,
                case.expect["variant"].as_str().expect("variant"),
                "case {}",
                case.id
            );
            if let AppMessage::Ev(event) = &message {
                assert_eq!(
                    app_event_variant(event),
                    case.expect["event"].as_str().expect("event variant"),
                    "case {}",
                    case.id
                );
            }
        }
        "app_message_invalid" => {
            assert!(
                serde_json::from_value::<AppMessage>(case.input.clone()).is_err(),
                "case {} should fail",
                case.id
            );
        }
        "app_event" => {
            let event: AppEvent =
                serde_json::from_value(case.input.clone()).expect("valid app event");
            assert_eq!(
                app_event_variant(&event),
                case.expect["variant"].as_str().expect("variant"),
                "case {}",
                case.id
            );
            let valid = case.expect["valid"].as_bool().expect("valid flag");
            assert_eq!(event.validate().is_ok(), valid, "case {}", case.id);
        }
        "device_operate" => {
            let operate: DeviceOperate =
                serde_json::from_value(case.input.clone()).expect("valid device operate");
            assert_eq!(
                action_name(operate.action),
                case.expect["action"].as_str().expect("action"),
                "case {}",
                case.id
            );
            assert_eq!(
                operate.validate().is_ok(),
                case.expect["valid"].as_bool().expect("valid flag"),
                "case {}",
                case.id
            );
        }
        "device_operate_invalid" => {
            assert!(
                serde_json::from_value::<DeviceOperate>(case.input.clone()).is_err(),
                "case {} should fail",
                case.id
            );
        }
        "clear_operate" => {
            let clear: ClearOperate =
                serde_json::from_value(case.input.clone()).expect("valid clear operate");
            assert_eq!(
                clear.validate().is_ok(),
                case.expect["valid"].as_bool().expect("valid flag"),
                "case {}",
                case.id
            );
        }
        "device_operate_result" => {
            let result: DeviceOperateResult =
                serde_json::from_value(case.input.clone()).expect("valid operation result");
            assert_eq!(
                action_name(result.action),
                case.expect["action"].as_str().expect("action"),
                "case {}",
                case.id
            );
            assert_eq!(
                serde_json::to_value(result.reason).expect("reason serialization"),
                case.expect["reason"],
                "case {}",
                case.id
            );
        }
        other => panic!("unknown V4 vector kind: {other}"),
    }
}
