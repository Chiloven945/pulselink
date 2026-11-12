# PawPrints V1.0 / V1.1 BLE

Summary of the wire behavior used by `pulselink`.

## GATT

- Control service: `0x180C`
- Write: `0x150A`
- Notify: `0x150B`
- Advertising names:
    - V1.1: `47L120300`
    - V1.0: `47L120100`

## Commands

- `0x50` — 17-byte trigger configuration.
    - Send a **no-trigger** configuration immediately after connecting.
    - Supports random reaction, probability, external voltage, and raw sensor modes.
- `0x5F` — reset the current trigger parameter.
- `0x60` — start automatic angle detection.
- `0x70` — shoulder-light color (steady or flashing).

## Notifications

- `0x51` — device information / battery.
- `0x5A` — trigger activated.
- `0x5B` — trigger deactivated.
- `0x5C` — trigger parameter changed.
- `0xD0` — raw sensor data.
- `0xF1` — automatically detected angle thresholds.

PawPrints is not exposed as a V4 remote device type.
