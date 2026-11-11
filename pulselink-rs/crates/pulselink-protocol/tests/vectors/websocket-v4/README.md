# DG-LAB V4 test vectors

Canonical JSON vectors for DG-LAB V4 WebSocket.

Planned files:

- `frames.json` — outer relay frames;
- `rpc.json` — request/response/event envelopes;
- `device-op.json` — all ActionTypes and completion results;
- `events.json` — snapshot/patch/slots.patch/custom.action;
- `invalid.json` — malformed JSON and invalid parameters.

The Rust copy lives in `pulselink-rs/crates/pulselink-protocol/tests/vectors/websocket-v4/`.
