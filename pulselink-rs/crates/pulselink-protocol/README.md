# pulselink-protocol

Sans-io DG-LAB protocol codecs for `pulselink`.

## Features

- `coyote-v3` (default) — Coyote V3 BLE binary codecs;
- `socket-v4` — DG-LAB V4 WebSocket JSON and RPC types.

The crate performs no I/O and does not depend on Tokio or btleplug.

## Coyote V3

```rust
use pulselink_core::{ChannelMap, Intensity};
use pulselink_pattern::{PulseIntensity, PulseInterval, PulseSample};
use pulselink_protocol::coyote::v3::{
    B0Command, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};

let command = B0Command {
    sequence: Sequence::new(1)?,
    intensity: StereoIntensityOp {
        a: IntensityOp::Set(Intensity::new(10)?),
        b: IntensityOp::Keep,
    },
    waveform: StereoWaveform {
        a: ChannelWaveform::Active([
            PulseSample::new(PulseInterval::new(100)?, PulseIntensity::new(20)?),
            PulseSample::new(PulseInterval::new(100)?, PulseIntensity::new(30)?),
            PulseSample::new(PulseInterval::new(100)?, PulseIntensity::new(20)?),
            PulseSample::new(PulseInterval::new(100)?, PulseIntensity::new(10)?),
        ]),
        b: ChannelWaveform::Silent,
    },
};

let bytes: [u8; 20] = command.encode();
# Ok::<(), Box<dyn std::error::Error>>(())
```

Also provides:

- `BfCommand::encode()` for 7-byte settings;
- `B1Message::decode()` for 4-byte intensity responses;
- `decode_battery()` for 1-byte battery notifications;
- `encode_interval()` / `decode_wire_frequency()` for pulse interval mapping.

## DG-LAB V4

With `socket-v4` enabled:

- `RelayFrame` — outer relay frames;
- `AppMessage` / `RpcRequest` / `RpcResponse` — inner RPC;
- `RemoteDevice` / `DevicePatch` / `SlotPatch` — device model;
- `AppEvent` — app events;
- `DeviceOperate` / `ClearOperate` / `DeviceOperateResult` — device operations.

## Test vectors

Canonical vectors live in `spec/test-vectors/` at the repository root. The crate-local copy is
synchronized by `scripts/sync-vectors.sh`.
