# pulselink

`pulselink` facade — the single entry point for most users.

```toml
[dependencies]
pulselink = "0.1"
```

Features:

- `ble` (default) — local BLE control;
- `websocket` — DG-LAB V4 controller;
- `full` — both.

The facade currently exposes:

- core domain types at the crate root;
- `pulselink::pattern`;
- `pulselink::protocol` when at least one protocol feature is enabled;
- `pulselink::transport`;
- `pulselink::ble` and `pulselink::runtime` with the `ble` feature;
- `pulselink::ws` with the `websocket` feature.

See `docs/ROADMAP.md`.
