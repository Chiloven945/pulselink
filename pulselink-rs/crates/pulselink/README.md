# pulselink

`pulselink` facade — the single entry point for most users.

```toml
[dependencies]
pulselink = "0.1"
```

Features:

- `ble` (default) — local BLE control for Coyote V2/V3, PawPrints, Civet, and Opossum;
- `websocket` — DG-LAB V4 controller via `Socket` and `AppClient`;
- `protocol` — sans-io protocol families without a transport;
- `full` — `ble` + `websocket`.

Supported local devices:

| Device                | BLE name                  |
|-----------------------|---------------------------|
| Coyote V2             | `D-LAB ESTIM01`           |
| Coyote V3             | `47L121000`               |
| PawPrints V1.1 / V1.0 | `47L120300` / `47L120100` |
| Civet V1              | `47L124000`               |
| Opossum V1            | `47L127000`               |

## BLE

`PulseLink::connect_device` auto-detects the device profile and returns a
`LocalSession`. `connect_coyote_v3` remains available for the V3 runtime API.

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

`PulseLink::with_backend` accepts any `Arc<dyn Backend>` for tests or custom transports. All session
behavior is delegated to `pulselink-device` and
`pulselink-runtime`.

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

[`Error`] wraps core, protocol, transport, BLE, runtime, and WebSocket errors behind one facade
type. Advanced users can still use the submodules and the individual crates directly.

## Examples

- `ble_scan` — discover Coyote V3 devices;
- `coyote_v3_control` — safe low-intensity hardware example (`PULSELINK_HW=1`);
- `coyote_v2_control` — low-intensity Coyote V2 example;
- `opossum_control` — Opossum vibration example;
- `civet_pressure` — Civet pressure example;
- `paw_prints_events` — PawPrints event example;
- `ws_pairing` — connect a V4 relay and display the target id;
- `ws_control` — list devices and submit safe operations;
- `v4_typed_devices` — print typed V4 snapshots;
- `protocol_only` — encode a B0 command without BLE or WebSocket.

See [the roadmap](https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/ROADMAP.md).
