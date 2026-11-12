# Documentation

This directory contains language-neutral documentation shared by the Rust and JVM implementations.

| Document | Purpose |
|---|---|
| [`architecture.md`](architecture.md) | Monorepo architecture and module boundaries |
| [`devices.md`](devices.md) | All supported DG-LAB devices and protocol notes |
| [`hardware-matrix.md`](hardware-matrix.md) | Real-device verification records |
| [`platforms.md`](platforms.md) | Linux, macOS, and Windows BLE notes |
| [`security.md`](security.md) | Pairing credentials, BLE output safety, dependency policy |
| [`testing.md`](testing.md) | Cross-language test strategy |
| [`local-relay.md`](local-relay.md) | Local DG-LAB V4 WebSocket relay |
| [`coyote-v3-privacy-broadcast.md`](coyote-v3-privacy-broadcast.md) | Coyote V3 unnamed-advertisement fallback |
| [`jvm.md`](jvm.md) | JVM implementation entry point |

Implementation-specific documentation stays with the implementation:

- Rust: [`pulselink-rs/docs/`](../pulselink-rs/docs/)
- JVM: [`pulselink4j/docs/`](../pulselink4j/docs/)

Protocol facts and cross-language test vectors live in [`spec/`](../spec/).
