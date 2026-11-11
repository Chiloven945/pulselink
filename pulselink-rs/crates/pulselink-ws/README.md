# pulselink-ws

DG-LAB V4 WebSocket controller SDK for `pulselink`.

The crate owns the controller-side WebSocket connection, pairing target id, multiple app clients,
RPC requests, device cache, and `device.op` lifecycle. It does not depend on BLE or the local-device
runtime.

## Usage

```rust,no_run
use std::time::Duration;

use pulselink_ws::{Channel, OperationOptions, Socket};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let socket = Socket::connect("wss://relay.example/v4").await?;
println!("pair target id: {}", socket.target_id());

let app = socket.wait_for_client().await?;
let devices = app.request_devices().await?;
let device = app.device(&devices[0].slot_id).ok_or("device removed")?;

device
    .add_intensity(Channel::A, 5, OperationOptions::default())
    .await?
    .completion()
    .await?;

device
    .set_temp_intensity(
        Channel::A,
        pulselink_ws::Intensity::new(20)?,
        Duration::from_secs(3),
        OperationOptions::default(),
    )
    .await?
    .completion()
    .await?;

socket.shutdown().await?;
# Ok(())
# }
```

## API

- `Socket` — connection, target id, state, app registry, lifecycle events.
- `AppClient` — device list/cache, app-level ping, clear, and live device handles.
- `RemoteDeviceHandle` — snapshot and typed operations for one slot.
- `OperationOptions` / `OperationHandle` — priority, immediate replacement, optional completion
  timeout, and completion results.
- `PulseFrame` — eight-byte version-3 pulse frame for `send_pulse`.
- `WsEvent` — app-scoped snapshot/patch/custom/disconnect events.
- `SocketEvent` — connection lifecycle, heartbeat, pong, close, and errors.

`device.op` responses arrive when a task completes, is cleared, replaced, or cancelled.
`AppendPulseData` and `SetTempIntensity` may stay pending for a long time; use
`OperationOptions::timeout` only when the caller needs a bound.

## Reconnect

A V4 controller receives a new `clientId`/target id for every connection. The SDK does not silently
reconnect or reuse an old target id. After a disconnect, create a new `Socket`, obtain the new
target id, and pair the app again.

## Testing

Enable the `mock` feature for `pulselink_ws::testing::MockRelay`:

```toml
pulselink-ws = { version = "0.1", features = ["mock"] }
```

The mock relay is a real localhost WebSocket server. It supports hello, client attach/detach, device
snapshots and patches, RPC success/error/timeout, `device.op` completion/clear, heartbeat/pong, idle
timeout, and close codes.
