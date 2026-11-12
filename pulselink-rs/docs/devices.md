# Local device support

`pulselink-device` adds local BLE drivers for every currently documented DG-LAB device.
`pulselink-ble` remains protocol-agnostic; the device crate owns profiles, commands, notifications,
and per-device sessions.

## Support matrix

| Device         | BLE name        | Session            | Status             |
|----------------|-----------------|--------------------|--------------------|
| Coyote V2      | `D-LAB ESTIM01` | `CoyoteV2Session`  | codec + mock tests |
| Coyote V3      | `47L121000`     | `CoyoteV3Session`  | 0.1.0 + runtime    |
| PawPrints V1.1 | `47L120300`     | `PawPrintsSession` | codec + mock tests |
| PawPrints V1.0 | `47L120100`     | `PawPrintsSession` | codec + mock tests |
| Civet V1       | `47L124000`     | `CivetSession`     | codec + mock tests |
| Opossum V1     | `47L127000`     | `OpossumSession`   | codec + mock tests |

Real-device verification is tracked in [`hardware-matrix.md`](hardware-matrix.md). Until a row is
verified there, the support level is codec/mock only.

## Discovery

`DeviceClient::connect` classifies by BLE name first, then by service UUID. This is required because
Coyote V3, PawPrints, Civet, and Opossum all use service
`0x180C`.

`DeviceProfile::scan_filter()` produces the matching name-prefix/service filter.

## Coyote V2

- Service `0x180B`; intensity `0x1504`; pulses `0x1505`/`0x1506`.
- 24-bit little-endian bitfields.
- Intensity `0..2047`; use `CoyoteV2Intensity`, not Coyote V3's `Intensity`.
- 3-byte `[X, Y, Z]` pulse parameters, valid for 100 ms.
- No B0/B1/BF or ACK. Use PWM_AB2 notifications to observe physical changes.

## Opossum

- `0x50` indicator/button reporting.
- `0xB0` 20-byte dual-channel vibration frames; 4 bytes per channel, every 25 ms.
- `0xB3` A/B intensity (`0xFF` = unchanged).
- `0xB2` display sync; only sent when negotiated MTU is at least 24.
- Notifications: `0xB3` intensity, `0xD0` button bitmask.

## Civet

- `0x50` starts/stops pressure reporting.
- `0x66` resets pressure and sets display orientation.
- `0xD0` 17-byte pressure notification; `i16` little-endian / 100 kPa.
- Use `Pressure::as_centi_kpa()` for exact values.

## PawPrints

- `0x50` 17-byte trigger configuration; **send a no-trigger config immediately after connecting**.
- `0x5F` reset parameter, `0x60` auto-detect angles, `0x70` shoulder lights.
- Notifications: `0x51` info/battery, `0x5A/0x5B/0x5C` trigger events,
  `0xD0` raw sensor data, `0xF1` angle ranges.

## V4 remote devices

Typed snapshots are available without a new wire protocol:

```rust
use pulselink::ws::TypedRemoteSnapshot;

match handle.typed_snapshot() {
TypedRemoteSnapshot::Coyote(snapshot) => { /* intensity/status */ }
TypedRemoteSnapshot::Opossum(snapshot) => { /* mode/connectivity */ }
TypedRemoteSnapshot::Civet(snapshot) => { /* pressure/edge state */ }
TypedRemoteSnapshot::Unknown(snapshot) => { /* forward-compatible */ }
}
```

Coyote V2 remote devices use V4 pulse version 2 (3-byte frames); Coyote V3 and Opossum use version 3
(8-byte frames). Civet does not accept pulse operations.

## Safety

- Never use `Intensity` (0..200) as a Coyote V2 wire intensity.
- Device drivers do not automatically restore output after reconnect.
- Opossum shutdown writes zero vibration and zero intensity.
- Civet shutdown stops pressure reporting.
- Hardware tests are opt-in with `PULSELINK_HW=1`; output additionally requires
  `PULSELINK_HW_OUTPUT=1`.
