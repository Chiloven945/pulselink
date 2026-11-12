# pulselink-relay

Local DG-LAB V4 WebSocket relay for testing `pulselink` without the official
relay.

This crate is not published. It is used by `pulselink-lab` and by integration
tests.

## Run

```bash
cargo run -p pulselink-relay -- --bind 0.0.0.0:8787
```

The relay exposes `ws://<host>:8787/v4`. A controller connects first and receives
`hello { clientId }`; that id is the app pairing target id. The app connects to:

```text
ws://<host>:8787/v4?tid=<targetId>
```

`pulselink-lab` can start the relay and display the pairing URL with `ws local`.

## Protocol

The relay implements the V4 controller/app topology:

- controller → relay: `ws://host:8787/v4`
- app → relay: `ws://host:8787/v4?tid=<targetId>`
- controller receives `hello` and `client_attached`
- app receives `controller_attached`
- messages are forwarded bidirectionally; the relay adds `clientId` for app →
  controller frames
- app disconnect sends `client_disconnected` to the controller
- controller disconnect sends `controller_disconnected` and close code 4000 to
  apps
- heartbeat every 30 seconds; ping receives pong
- optional idle timeout closes an unpaired controller with code 4002

## Security

- The target id is a pairing secret. Do not log it in production.
- The relay listens on the LAN and has no authentication or TLS. Use it only on
  a trusted network.
- Some DG-LAB app versions may require `wss://` or a trusted certificate. Put the
  relay behind a TLS tunnel or reverse proxy if plain `ws://` is rejected.
