# Architecture

`pulselink` is a multi-language SDK for DG-LAB devices. Rust and JVM share protocol facts and test
vectors, but each implementation follows its language's idioms and build system.

## Repository graph

```mermaid
flowchart TD
    spec[spec/ protocol facts + vectors]
    docs[docs/ language-neutral docs]

    subgraph Rust[pulselink-rs]
        rcore[pulselink-core]
        rprotocol[pulselink-protocol]
        rtransport[pulselink-transport]
        rble[pulselink-ble]
        rdevice[pulselink-device]
        rruntime[pulselink-runtime]
        rws[pulselink-ws]
        rrelay[pulselink-relay]
        rfacade[pulselink]
        rlab[pulselink-lab]
    end

    subgraph JVM[pulselink4j]
        jcore[pulselink-core]
        jprotocol[pulselink-protocol]
        jtransport[pulselink-transport]
        jble[pulselink-ble]
        jdevice[pulselink-device]
        jruntime[pulselink-runtime]
        jws[pulselink-ws]
        jjava[pulselink-java]
        jtesting[pulselink-testing]
    end

    rprotocol --> rcore
    rtransport --> rcore
    rble --> rtransport
    rdevice --> rcore
    rdevice --> rprotocol
    rdevice --> rtransport
    rdevice --> rruntime
    rruntime --> rcore
    rruntime --> rprotocol
    rruntime --> rtransport
    rws --> rcore
    rws --> rprotocol
    rrelay --> rprotocol
    rfacade --> rcore
    rfacade --> rprotocol
    rfacade --> rtransport
    rfacade --> rruntime
    rfacade --> rble
    rfacade --> rdevice
    rfacade --> rws
    rlab --> rfacade
    rlab --> rrelay
    rlab --> rws

    jprotocol --> jcore
    jtransport --> jcore
    jble --> jtransport
    jdevice --> jcore
    jdevice --> jprotocol
    jdevice --> jtransport
    jdevice --> jruntime
    jruntime --> jcore
    jruntime --> jprotocol
    jruntime --> jtransport
    jws --> jcore
    jws --> jprotocol
    jjava --> jruntime
    jjava --> jws
    jjava --> jble
    jtesting --> jcore
    jtesting --> jprotocol
    jtesting --> jtransport

    rprotocol -. test vectors .-> spec
    jprotocol -. test vectors .-> spec
    Rust -. shared docs .-> docs
    JVM -. shared docs .-> docs
```

## Boundaries

- `pulselink-core` is synchronous domain state; no I/O.
- `pulselink-protocol` is sans-io; no BLE, WebSocket, or runtime dependencies.
- `pulselink-transport` is the local-device port; platform backends implement it.
- `pulselink-ble` is a platform adapter. Rust uses btleplug; JVM uses Kable.
- `pulselink-device` owns device profiles and local device sessions.
- `pulselink-runtime` owns Coyote V3 session behavior, ACK, scheduler, safety, and reconnect.
- `pulselink-ws` owns DG-LAB V4 WebSocket RPC, multi-app state, and remote operations.
- `pulselink` (Rust) and `pulselink-java` (JVM) are composition/facade layers only.
- `pulselink-relay`, `pulselink-lab`, and `pulselink-testing` are non-published development tools.

## Shared specification

`spec/` is the canonical source for protocol behavior and test vectors. Implementations may keep
test-only copies for their build system, but CI must verify those copies match `spec/`.

## Language-specific references

- Rust architecture: [`pulselink-rs/docs/architecture.md`](../pulselink-rs/docs/architecture.md)
- JVM roadmap: [`pulselink4j/ROADMAP.md`](../pulselink4j/ROADMAP.md)
- JVM build/nullness notes: [`pulselink4j/docs/`](../pulselink4j/docs/)
