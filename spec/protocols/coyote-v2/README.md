# Coyote V2 BLE

Reference: <https://github.com/dungeonlab-open/dglab-bluetooth-protocol/tree/main/coyote/v2>

Summary of the wire behavior used by `pulselink`; this is not a copy of the official text.

## GATT

- PWM service: `0x180B`
- A/B intensity: `0x1504` (`PWM_AB2`)
- A-channel pulse: `0x1505` (`PWM_A34`)
- B-channel pulse: `0x1506` (`PWM_B34`)
- Battery service: `0x180A`
- Battery level: `0x1500`
- Advertising name: `D-LAB ESTIM01`

## Semantics

- Intensity is `0..2047`, not the Coyote V3 `0..200` range.
- `PWM_AB2` carries two 11-bit intensity fields.
- Pulse parameters are 3 bytes: `[X, Y, Z]`, valid for a 100 ms interval.
- 24-bit bitfields are little-endian.
- There is no B0/B1/BF command and no ACK.
- `PWM_AB2` notifications report physical intensity changes.

Implementations must use a distinct V2 intensity type; do not reuse the V3 `Intensity` type.
