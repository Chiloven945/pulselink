# Civet V1 BLE

Summary of the wire behavior used by `pulselink`.

## GATT

- Control service: `0x180C`
- Write: `0x150A`
- Notify: `0x150B`
- Battery service: `0x180A` / `0x1500`
- Advertising name: `47L124000`

## Commands

- `0x50` — start/stop pressure reporting.
- `0x66` — reset pressure and set display orientation.

## Notifications

- `0xD0` — pressure report.
    - 17-byte message.
    - Pressure samples are signed 16-bit little-endian values.
    - Wire unit is `1/100 kPa`; preserve exact fixed-point values.

Civet does not accept V4 pulse operations.
