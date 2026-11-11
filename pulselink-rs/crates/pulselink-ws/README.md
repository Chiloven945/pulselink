# pulselink-ws

DG-LAB V4 WebSocket controller SDK for pulselink-rs.

Planned API:

- `Socket` — controller connection, `targetId`, app registry;
- `AppClient` — one connected DG-LAB app;
- `RemoteDeviceHandle` — device operations;
- `OperationHandle` — submit/completion semantics;
- RPC manager, device cache, multi-app support.

This crate does not depend on BLE or the runtime.

Implementation starts in milestone M5/M7. See `docs/ROADMAP.md` and `docs/websocket-v4.md`.
