# Opossum V1 BLE

Summary of the wire behavior used by `pulselink`.

## GATT

- Control service: `0x180C`
- Write: `0x150A`
- Notify: `0x150B`
- Battery service: `0x180A` / `0x1500`
- Advertising name: `47L127000`

## Commands

- `0x50` — indicator color and button reporting.
- `0xB0` — 20-byte dual-channel vibration frame.
    - Four bytes per channel.
    - Intended update interval: 25 ms.
- `0xB3` — A/B intensity.
    - Values `0..200`.
    - `0xFF` means "leave unchanged".
- `0xB2` — 24-byte display intensity sync.
    - Only send when the negotiated MTU is at least 24.

## Notifications

- `0xB3` — intensity report.
- `0xD0` — button bitmask.

Shutdown should write zero vibration and zero intensity.
