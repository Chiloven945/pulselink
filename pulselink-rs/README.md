# pulselink-rs

The Rust workspace for `pulselink`, a reliable, asynchronous SDK for DG-LAB devices.

`pulselink-rs` contains a small facade crate, `pulselink`, and explicit internal boundaries:

| Crate                 | Responsibility                                   |
|-----------------------|--------------------------------------------------|
| `pulselink-core`      | Domain types, state, events, errors              |
| `pulselink-pattern`   | Waveform timeline types                          |
| `pulselink-protocol`  | Sans-io Coyote V3 BLE and DG-LAB V4 codecs       |
| `pulselink-transport` | Local device transport SPI                       |
| `pulselink-ble`       | btleplug backend                                 |
| `pulselink-runtime`   | Session actor, ACK, scheduler, safety, reconnect |
| `pulselink-ws`        | DG-LAB V4 WebSocket controller                   |
| `pulselink`           | Facade                                           |

## Status

Milestones **M0 (workspace foundation)**, **M1 (core + pattern)**, **M2 (protocol)**, **M3 (
transport)**, **M4 (BLE backend)**, and **M5 (WebSocket transport/RPC)** are implemented. Runtime,
full WebSocket session API, and facade finalization follow the plan in
[`docs/ROADMAP.md`](docs/ROADMAP.md).

## Quick start

The published API is not ready yet. For now:

```bash
cargo test --workspace
cargo doc --workspace --no-deps
```

## Scope for 0.1

- Coyote V3 BLE;
- DG-LAB V4 WebSocket;
- Windows, Linux, macOS first.

Not in 0.1: Coyote V2 BLE, V3 WebSocket, PawPrints/Civet/Opossum drivers, audio/graph/CLI/FFI,
Android/iOS first-class support.

## License and protocol notice

The source code of pulselink-rs is licensed under Apache-2.0. See [`LICENSE`](LICENSE).

The DG-LAB Bluetooth protocol documentation states that commercial use of the protocol content
requires prior authorization from DG-LAB. The Apache-2.0 license covers the code of pulselink-rs
only; it does not grant rights to DG-LAB protocols, trademarks, or device firmware. Commercial
projects should confirm authorization with DG-LAB.

## Documentation

- [`docs/ROADMAP.md`](docs/ROADMAP.md) — full implementation roadmap and task list
- `docs/architecture.md` — crate graph and boundaries
- `docs/runtime-contracts.md` — session/ACK/scheduler contracts
- `docs/testing.md` — test strategy
