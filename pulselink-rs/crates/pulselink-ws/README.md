# pulselink-ws

DG-LAB V4 WebSocket controller SDK for `pulselink`.

The crate owns the controller-side WebSocket connection, pairing target id, multiple app clients,
RPC requests, device cache, and `device.op` submission. It does not depend on BLE or the
local-device runtime.

## Usage

```rust,no_run
use pulselink_ws::{ClearTarget, Socket};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let socket = Socket::connect("wss://relay.example/v4").await?;
println!("pair target id: {}", socket.target_id());

let client = socket.wait_for_client().await?;
let devices = client.request_devices().await?;
println!("{} device(s)", devices.len());

client.clear_operate(ClearTarget::All).await?;
socket.shutdown().await?;
# Ok(())
# }
```

## API

- `Socket` — connection, `target_id`, state, app registry, lifecycle events.
- `SocketOptions` — URL, connect/response timeouts, optional ping interval, max message size.
- `AppClient` — device list/cache, app-level ping, `device.op.clear`, and raw
  `device.op` submission.
- `PendingOperation` — waits for a `device.op` completion and returns the
  `DeviceOperateResult`.
- `WsEvent` — app-scoped snapshot/patch/custom/disconnect events.
- `SocketEvent` — connection lifecycle, heartbeat, pong, close, and errors.

`device.op` responses arrive when a task completes, is cleared, replaced, or cancelled.
`AppendPulseData` and `SetTempIntensity` may therefore stay pending for a long time. Pass `None` as
the operation timeout when the task duration is controlled by `duration_ms` or by `device.op.clear`.

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
snapshots and patches, RPC success/error/timeout,
`device.op` completion/clear, heartbeat/pong, and close codes.
