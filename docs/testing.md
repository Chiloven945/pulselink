# Testing

The full Rust strategy is in [`pulselink-rs/docs/ROADMAP.md`](../pulselink-rs/docs/ROADMAP.md). This page is the cross-implementation quick reference.

## Layers

| Layer          | Location                                  | Hardware |
|----------------|-------------------------------------------|----------|
| Unit           | crate `src` tests                         | No       |
| Golden vectors | `pulselink-protocol/tests/vectors`        | No       |
| Property       | `pulselink-protocol/tests`                | No       |
| Mock GATT      | `pulselink-runtime` / `integration-tests` | No       |
| Local relay    | `pulselink-relay` / `pulselink-lab`       | No       |
| Terminal UI    | `pulselink-lab` (`TestBackend`, worker)   | No       |
| Hardware       | `integration-tests/tests/hardware.rs`     | Yes      |
| JVM unit       | `pulselink4j/*/src/test`                   | No       |
| JVM WebSocket  | Ktor test server                           | No       |
| JVM hardware   | `PULSELINK_HW=1` gated tests               | Yes      |

## Commands

Rust:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo test -p pulselink-protocol --features socket-v4
cargo test -p pulselink-relay
cargo test -p pulselink-lab
cargo doc --workspace --no-deps
cargo run -p pulselink-lab
```

JVM:

```bash
cd pulselink4j
./gradlew check
./gradlew apiCheck
./gradlew dokkaHtml
```

## Test vectors

Canonical vectors live in `spec/test-vectors/` at the repository root, shared with `pulselink4j`.
Rust tests use a copy inside `pulselink-protocol` because Cargo packages cannot include files
outside the crate directory. CI must verify the copies are in sync.

## Hardware tests

Hardware tests are `#[ignore]` by default and gated by environment variables:

```bash
PULSELINK_HW=1 cargo test -p pulselink-integration-tests --test hardware -- --ignored
```

The default hardware probe only scans, connects, reads battery, subscribes, and writes a low BF soft
limit. Actual stimulation requires an explicit opt-in.

`pulselink-lab` is the interactive test bench. Output is disabled by default; press `Space` to
enable it, `F12` for emergency stop, and `:` for commands. The worker can be driven by `MockBackend`
tests without a terminal, and the UI is rendered with `ratatui::backend::TestBackend` in unit tests.
