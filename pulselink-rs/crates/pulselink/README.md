# pulselink

`pulselink` facade — the single entry point for most users.

```toml
[dependencies]
pulselink = "0.1"
```

Features:

- `ble` (default) — local BLE control via `PulseLink` and the Coyote V3 runtime;
- `websocket` — DG-LAB V4 controller via `Socket` and `AppClient`;
- `protocol` — both sans-io protocol families without a transport;
- `full` — `ble` + `websocket`.

## BLE

```rust,no_run
use std::time::Duration;

use pulselink::protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink::transport::ScanFilter;
use pulselink::PulseLink;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let link = PulseLink::new().await?;
let devices = link
    .scan(
        ScanFilter {
            name_prefixes: vec![DEVICE_NAME.to_owned()],
            service_uuids: vec![SERVICE_CONTROL],
        },
        Duration::from_secs(5),
    )
    .await?;
if let Some(device) = devices.first() {
    let session = link.connect_coyote_v3(device).await?;
    let _state = session.state();
    session.shutdown().await?;
}
link.shutdown().await;
# Ok(())
# }
```

`PulseLink::with_backend` accepts any `Arc<dyn Backend>` for tests or custom
transports. All session behavior is delegated to `pulselink-runtime`.

## WebSocket

```rust,no_run
use pulselink::ws::Socket;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let socket = Socket::connect("wss://relay.example/v4").await?;
println!("pair target id: {}", socket.target_id());
let client = socket.wait_for_client().await?;
let devices = client.request_devices().await?;
println!("{} device(s)", devices.len());
socket.shutdown().await?;
# Ok(())
# }
```

## Error

[`Error`] wraps core, protocol, transport, BLE, runtime, and WebSocket errors
behind one facade type. Advanced users can still use the submodules and the
individual crates directly.

## Examples

- `ble_scan` — discover Coyote V3 devices;
- `coyote_v3_control` — safe low-intensity hardware example (`PULSELINK_HW=1`);
- `ws_pairing` — connect a V4 relay and display the target id;
- `ws_control` — list devices and submit safe operations;
- `protocol_only` — encode a B0 command without BLE or WebSocket.

See [the roadmap](https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/ROADMAP.md).
