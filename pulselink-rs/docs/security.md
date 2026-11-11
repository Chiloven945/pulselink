# Security

## Scope

`pulselink` is a client SDK. It does not implement a relay, account system, or network
authentication service. Security properties depend on the DG-LAB app, relay, and Bluetooth stack in
use.

## Pairing credentials

A V4 `targetId` is a pairing credential. Treat it like a short-lived secret:

- do not log the full target id at `info` or `debug`;
- do not include it in crash reports;
- do not persist it across controller reconnects;
- display it only to the user who is pairing an app.

The SDK does not silently reconnect or reuse an old target id. A new controller connection receives
a new id and requires a new app pairing.

## BLE output safety

`pulselink-runtime` applies a `SafetyPolicy` before intensity operations and sends a best-effort
zero-intensity B0 during shutdown. A dropped BLE connection can still leave device-side state;
callers should use conservative soft limits and provide an independent physical stop path where
required.

## Memory and dependency safety

- The workspace forbids `unsafe_code`.
- `cargo deny check` runs in CI for advisories, bans, licenses, and sources.
- TLS uses rustls with WebPKI roots; certificate verification is enabled.
- The protocol crates are sans-io and do not perform network or BLE I/O.

## Protocol notice

The DG-LAB Bluetooth protocol documentation states that commercial use of the protocol content
requires prior authorization from DG-LAB. The Apache-2.0 license covers the source code only; it
does not grant rights to DG-LAB protocols, trademarks, or device firmware.

## Reporting

Report security issues privately to the repository maintainers rather than in a public issue.
Include the affected crate/version, a minimal reproduction, and the expected impact.
