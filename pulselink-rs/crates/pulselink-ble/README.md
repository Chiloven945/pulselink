# pulselink-ble

btleplug-based local BLE backend for pulselink-rs.

This crate implements `pulselink-transport` and keeps btleplug types out of the
public API. It does not know Coyote V3 `B0`/`BF` semantics.

Implementation starts in milestone M4. See `docs/ROADMAP.md` and
`docs/platforms.md`.
