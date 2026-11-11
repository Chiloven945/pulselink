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

Milestones **M0 (workspace foundation)** through **M8 (facade)** are implemented. Release hardening
(M9) is in progress; see [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Quick start

```bash
cargo test --workspace
cargo doc --workspace --no-deps
cargo run -p pulselink --example ble_scan
cargo run -p pulselink --no-default-features --features protocol --example protocol_only
```

The facade is `pulselink`; use `PulseLink` for local BLE sessions and
`pulselink::ws::Socket` for DG-LAB V4 WebSocket control.

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

## Security

See [`docs/security.md`](docs/security.md) for pairing-credential handling, BLE output safety, and
the protocol notice.

## Documentation

- [`docs/ROADMAP.md`](docs/ROADMAP.md) — full implementation roadmap and task list
- `docs/architecture.md` — crate graph and boundaries
- `docs/runtime-contracts.md` — session/ACK/scheduler contracts
- `docs/testing.md` — test strategy
- `docs/security.md` — pairing, output-safety, and dependency notes
- `docs/hardware-matrix.md` — real-device verification procedure and results
- `docs/release.md` — crates.io release procedure and rate-limit notes
