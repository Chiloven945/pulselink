# Testing

The full strategy is in [`ROADMAP.md`](ROADMAP.md). This page is the quick reference.

## Layers

| Layer          | Location                                  | Hardware |
|----------------|-------------------------------------------|----------|
| Unit           | crate `src` tests                         | No       |
| Golden vectors | `pulselink-protocol/tests/vectors`        | No       |
| Property       | `pulselink-protocol/tests`                | No       |
| Mock GATT      | `pulselink-runtime` / `integration-tests` | No       |
| Mock relay     | `pulselink-ws` / `integration-tests`      | No       |
| Hardware       | `integration-tests/tests/hardware.rs`     | Yes      |

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo test -p pulselink-protocol --features socket-v4
cargo doc --workspace --no-deps
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
