# pulselink

`pulselink` is a multi-language SDK for DG-LAB devices.

| Implementation                  | Language                              | Status                         |
|---------------------------------|---------------------------------------|--------------------------------|
| [`pulselink-rs`](pulselink-rs/) | Rust                                  | `0.1.0` published on crates.io |
| [`pulselink4j`](pulselink4j/)   | Kotlin/JVM + optional Java extension  | J0-J9 complete                  |
| [`spec/`](spec/)                | Protocol specification + test vectors | Shared source of truth         |

## Repository layout

```text
pulselink/
├── pulselink-rs/        Rust workspace and crates
├── pulselink4j/         Gradle Kotlin/JVM workspace
├── spec/                Protocol summaries and cross-language test vectors
├── docs/                Language-neutral documentation
└── .github/workflows/   Rust and JVM CI
```

## What is supported

- Coyote V2 / V3 local BLE;
- PawPrints V1.0 / V1.1 local BLE;
- Civet V1 local BLE;
- Opossum V1 local BLE;
- DG-LAB V4 WebSocket remote devices: `COYOTE_020`, `COYOTE_030`, `BMTR_1`, `OVC_1`;
- Windows, Linux, and macOS first.

See [`docs/devices.md`](docs/devices.md) for the support matrix.

## Rust

```bash
cd pulselink-rs
cargo test --workspace --all-features
cargo run -p pulselink --example ble_scan
cargo run -p pulselink-lab
```

The published facade is the [`pulselink`](https://crates.io/crates/pulselink) crate. See
[`pulselink-rs/README.md`](pulselink-rs/README.md) for details.

## JVM

```bash
cd pulselink4j
./gradlew check
```

`pulselink4j` uses Kotlin 2.4.21, Gradle 9.9.0, Java 21 bytecode, JSpecify, Kable for BLE, and Ktor
for WebSocket. The Java extension is optional. See [`pulselink4j/README.md`](pulselink4j/README.md)
and [`pulselink4j/ROADMAP.md`](pulselink4j/ROADMAP.md).

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — monorepo architecture
- [`docs/devices.md`](docs/devices.md) — device support matrix
- [`docs/hardware-matrix.md`](docs/hardware-matrix.md) — real-device verification
- [`docs/platforms.md`](docs/platforms.md) — BLE platform notes
- [`docs/security.md`](docs/security.md) — pairing, output safety, dependencies
- [`docs/testing.md`](docs/testing.md) — cross-language test strategy
- [`docs/local-relay.md`](docs/local-relay.md) — local V4 relay
- [`docs/coyote-v3-privacy-broadcast.md`](docs/coyote-v3-privacy-broadcast.md) — V3
  unnamed-advertisement fallback
- [`spec/README.md`](spec/README.md) — protocol specification and vectors

## License and protocol notice

The source code is licensed under Apache-2.0. See [`LICENSE`](LICENSE).

The DG-LAB Bluetooth protocol documentation states that commercial use of the protocol content
requires prior authorization from DG-LAB. The Apache-2.0 license covers only this repository's code;
it does not grant rights to DG-LAB protocols, trademarks, or device firmware. Commercial projects
should confirm authorization with DG-LAB.
