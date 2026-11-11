# ADR 0001: Workspace and crate split

## Status

Accepted.

## Context

`pulselink-rs` is the Rust workspace for `pulselink`, a long-lived SDK for DG-LAB devices. It must support local BLE control and
DG-LAB V4 WebSocket control, keep protocol code independent of I/O, and allow third-party BLE
backends.

## Decision

Use a virtual Cargo workspace with eight library crates:

- `pulselink-core`
- `pulselink-pattern`
- `pulselink-protocol`
- `pulselink-transport`
- `pulselink-ble`
- `pulselink-runtime`
- `pulselink-ws`
- `pulselink` (facade)

Key choices:

1. The facade lives at `crates/pulselink`; the workspace root is virtual.
2. `pulselink-transport` exists from day one. It is a port, not a speculative abstraction.
3. `pulselink-runtime` depends on transport, not btleplug.
4. `pulselink-ws` is independent of BLE and runtime.
5. `pulselink-protocol` is sans-io.
6. A separate `pulselink-device` crate is deferred until a second device with a different control
   model exists.
7. All library crates are published together at the same version; the integration-test package is
   `publish = false`.
8. Coyote V3 BLE and DG-LAB V4 WebSocket are both in 0.1; Coyote V2 BLE and V3 WebSocket are not.
9. MSRV starts at Rust 1.99 (current stable at the time of the decision).
10. License is Apache-2.0. DG-LAB protocol commercial use is a separate concern.

## Consequences

- Users add one dependency, `pulselink`; Cargo resolves the internal crates as transitive
  dependencies.
- `cargo publish --workspace` publishes the release set.
- Third parties can implement `pulselink-transport` without depending on btleplug.
- Runtime tests can use the mock transport without compiling platform BLE code.
- More crates must be versioned and documented together.

## Alternatives rejected

- Single crate: couples platform BLE and WebSocket dependencies; poor feature isolation and
  testability.
- `pulselink-transport` merged into `pulselink-ble`: runtime would depend on the adapter crate,
  making backend replacement a breaking change.
- Separate `pulselink-device` now: only one device driver exists; the interface would be
  speculative.
- One protocol crate per device: unnecessary split at 0.1; feature flags are sufficient.
