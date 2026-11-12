# pulselink-rs

The Rust workspace for `pulselink`, a reliable, asynchronous SDK for DG-LAB devices.

`pulselink-rs` contains a small facade crate, `pulselink`, and explicit internal boundaries:

| Crate                 | Responsibility                                             |
|-----------------------|------------------------------------------------------------|
| `pulselink-core`      | Domain types, state, events, errors                        |
| `pulselink-pattern`   | Waveform timeline types                                    |
| `pulselink-protocol`  | Sans-io Coyote V3 BLE and DG-LAB V4 codecs                 |
| `pulselink-transport` | Local device transport SPI                                 |
| `pulselink-ble`       | btleplug backend                                           |
| `pulselink-runtime`   | Coyote V3 session actor, ACK, scheduler, safety, reconnect |
| `pulselink-device`    | Coyote V2, Opossum, Civet, PawPrints drivers               |
| `pulselink-ws`        | DG-LAB V4 WebSocket controller                             |
| `pulselink`           | Facade                                                     |

## Status

Rust 0.1.0 is published on crates.io. The current workspace adds the new
`pulselink-device` crate and local support for Coyote V2, PawPrints, Civet, and Opossum, plus typed
V4 remote-device snapshots. Real-hardware verification for the new devices is still in progress. See
[`docs/ROADMAP.md`](docs/ROADMAP.md).

## Quick start

```toml
[dependencies]
pulselink = "0.1.0"
```

```bash
cargo test --workspace
cargo doc --workspace --no-deps
cargo run -p pulselink --example ble_scan
cargo run -p pulselink --no-default-features --features protocol --example protocol_only
```

The facade is `pulselink`; use `PulseLink` for local BLE sessions and
`pulselink::ws::Socket` for DG-LAB V4 WebSocket control.

## Device support

- Coyote V2 / V3 local BLE;
- PawPrints V1.0 / V1.1 local BLE;
- Civet V1 local BLE;
- Opossum V1 local BLE;
- DG-LAB V4 remote devices: `COYOTE_020`, `COYOTE_030`, `BMTR_1`, `OVC_1`;
- Windows, Linux, macOS first.

Not yet included: legacy V3 WebSocket compatibility, audio/graph/CLI/FFI, Android/iOS first-class
support.

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
