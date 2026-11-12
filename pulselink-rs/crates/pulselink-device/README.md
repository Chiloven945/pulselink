# pulselink-device

Local DG-LAB device drivers for `pulselink`.

This crate adds support for devices that do not share the Coyote V3 control model. It uses the
protocol-agnostic `pulselink-ble` backend and keeps device knowledge out of the transport layer.

## Devices

| Device         | BLE name        | Session                                   |
|----------------|-----------------|-------------------------------------------|
| Coyote V2      | `D-LAB ESTIM01` | `CoyoteV2Session`                         |
| Coyote V3      | `47L121000`     | `CoyoteV3Session` via `pulselink-runtime` |
| PawPrints V1.1 | `47L120300`     | `PawPrintsSession`                        |
| PawPrints V1.0 | `47L120100`     | `PawPrintsSession`                        |
| Civet V1       | `47L124000`     | `CivetSession`                            |
| Opossum V1     | `47L127000`     | `OpossumSession`                          |

## Usage

```rust,no_run
use std::sync::Arc;
use std::time::Duration;

use pulselink_ble::BtleplugCentral;
use pulselink_device::{DeviceClient, LocalSession};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let backend = Arc::new(BtleplugCentral::new().await?);
let client = DeviceClient::new(backend);

let devices = client
    .scan(pulselink_transport::ScanFilter::default(), Duration::from_secs(5))
    .await?;
let session = client.connect(&devices[0]).await?;
match session {
    LocalSession::CoyoteV2(session) => {
        let intensity = pulselink_protocol::coyote::v2::CoyoteV2Intensity::from_display(5)?;
        session.set_intensity(pulselink_core::Channel::A, intensity).await?;
        session.shutdown().await?;
    }
    other => other.shutdown().await?,
}
# Ok(())
# }
```

## Design

- `DeviceClient::connect` classifies by BLE name first, then by service UUID.
- `DeviceClient::scan_profile` retries with the profile service UUID when no named device is found,
  covering Coyote V3 privacy-broadcast firmware. An unnamed `0x180C` device is a V3 candidate, not a
  guaranteed identification; see [
  `docs/coyote-v3-privacy-broadcast.md`](https://github.com/Chiloven945/pulselink/blob/main/docs/coyote-v3-privacy-broadcast.md).
- Each session owns its own notification pump and state.
- Coyote V3 delegates to `pulselink-runtime`; it is not reimplemented here.
- Sensor sessions expose typed methods (`pressure`, `snapshot`, trigger configuration) instead of
  pretending to be stimulators.
- Opossum display sync is sent only when the negotiated MTU can carry the 24-byte `0xB2` command.

## Feature flags

The protocol features are enabled by this crate:

```text
coyote-v2, coyote-v3, opossum, civet, paw-prints
```
