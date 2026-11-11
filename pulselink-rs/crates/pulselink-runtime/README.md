# pulselink-runtime

Device session runtime for pulselink-rs.

Planned components:

- `Client` — scan/connect;
- `CoyoteV3Session` — device session handle;
- Session Actor, command queue, state cache;
- Sequence/ACK manager;
- 100 ms scheduler and pattern player;
- safety policy, reconnect, shutdown.

The runtime depends on `pulselink-transport`, not btleplug.

Implementation starts in milestone M6. See `docs/ROADMAP.md` and `docs/runtime-contracts.md`.
