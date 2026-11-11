# pulselink-ble

btleplug-based local BLE backend for `pulselink`.

`pulselink-ble` implements the `pulselink-transport` traits and keeps btleplug types out of the
public API. It does not know Coyote `B0`/`BF` semantics; the runtime supplies the protocol bytes and
GATT UUIDs.

## Feature

The `btleplug` feature is enabled by default. Disable default features to compile the crate without
a platform BLE dependency.

## Usage

```rust,no_run
use std::time::Duration;

use pulselink_ble::BtleplugCentral;
use pulselink_transport::{Central, GattLink, ScanFilter, WriteMode};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let central = BtleplugCentral::new().await?;
let devices = central
    .scan(
        ScanFilter {
            name_prefixes: vec!["47L".to_owned()],
            ..ScanFilter::default()
        },
        Duration::from_secs(5),
    )
    .await?;

if let Some(device) = devices.first() {
    let link = central.connect(device).await?;
    // Write protocol bytes to the characteristic supplied by the runtime.
    link.write(
        uuid::Uuid::from_u128(0x150A),
        &[0xBF, 0, 0, 128, 128, 128, 128],
        WriteMode::Auto,
    )
    .await?;
    link.disconnect().await?;
}
# Ok(())
# }
```

## Behavior

- `BtleplugCentral::new()` selects the first adapter and returns
  `BleError::NoAdapter` for an empty list.
- Scans pass service UUIDs to btleplug and apply the SDK `ScanFilter` as a second pass. Results are
  deduplicated by device id.
- Dropping a scan future stops the underlying scan.
- `WriteMode::Auto` prefers write-without-response, then write-with-response.
- Each link shares one btleplug notification stream and fans notifications out to independent
  subscribers.
- Reconnect tries the known peripheral first and rescans with the stored filter when needed. It does
  not restore BF or output intensity.

## Hardware probe

```text
PULSELINK_HW=1 cargo run -p pulselink-ble --example coyote_v3_probe
```

The probe reads battery state, subscribes to notifications, writes a zero soft-limit BF, and sends a
zero-intensity B0. It never sends a non-zero intensity by default.

See [`docs/platforms.md`](https://github.com/Chiloven945/pulselink/blob/main/pulselink-rs/docs/platforms.md) for Linux, macOS, and Windows notes.
