# DG-LAB V4 WebSocket

References:

- <https://github.com/dungeonlab-open/dglab-kit>
- <https://github.com/dungeonlab-open/dglab-websocket-server>

Files:

- `frames.md` — outer relay frames;
- `rpc.md` — inner request/response/event envelopes;
- `device-op.md` — device operations and completion semantics.

The relay only forwards application messages; it does not parse device commands.
