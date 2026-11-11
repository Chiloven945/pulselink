# pulselink-runtime

Coyote V3 session runtime for `pulselink`.

The runtime owns the session actor, sequence/ACK handling, the 100 ms scheduler, safety policy,
reconnection, and shutdown. It depends on `pulselink-transport`, not on btleplug, so it can be
tested with the mock backend.

## Example

```rust,no_run
use std::sync::Arc;
use std::time::Duration;

use pulselink_core::{Channel, Intensity};
use pulselink_runtime::{Client, CoyoteV3Config};
use pulselink_transport::{Backend, ScanFilter};

# async fn example(backend: Arc<dyn Backend>) -> Result<(), Box<dyn std::error::Error>> {
let client = Client::new(backend);
let devices = client
    .scan(ScanFilter::default(), Duration::from_secs(5))
    .await?;
let device = devices.first().ok_or("no device")?;
let session = client
    .connect_coyote_v3(device, CoyoteV3Config::default())
    .await?;

session
    .set_intensity(Channel::A, Intensity::new(10)?)
    .await?;
session.stop().await?;
session.shutdown().await?;
client.shutdown().await;
# Ok(())
# }
```

## Behavior

- Initialization: subscribe notify, optional battery, read battery, write BF, then enter `Ready`.
- Intensity changes use B0 sequence `1..=15` and wait for matching B1.
- Waveform and silence frames are emitted every 100 ms; stale frames are skipped.
- `SafetyPolicy` and BF soft limits both apply; the stricter limit wins.
- Reconnect reinitializes BF but never restores a pattern or intensity.
- `shutdown().await` performs a best-effort zero-intensity write before closing.
- `Drop` only sends a cancellation signal; use `shutdown` for deterministic cleanup.

See [`https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/runtime-contracts.md`](https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/runtime-contracts.md) and
[`https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/ROADMAP.md`](https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/ROADMAP.md).
