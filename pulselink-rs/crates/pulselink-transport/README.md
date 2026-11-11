# pulselink-transport

Local device transport SPI for `pulselink`.

This crate defines the port used by `pulselink-runtime`. It has no Tokio or
btleplug dependency; platform backends implement its traits.

## Traits

```rust
use pulselink_transport::{Backend, Central, Connector, GattLink};
```

- `GattLink` — write/read/subscribe/disconnect one characteristic set;
- `Central` — scan and connect;
- `Connector` — reconnect to a known device;
- `Backend` — `Central + Connector`.

## Types

- `TransportDeviceId` — opaque platform device id;
- `DiscoveredDevice` — scan result;
- `ScanFilter` — name-prefix and service-UUID filter;
- `WriteMode` — auto/with-response/without-response.

`ScanFilter::matches` uses OR semantics: an empty filter matches everything;
otherwise a name-prefix match or a service-UUID match is sufficient.

## Mock

Enable the `mock` feature for deterministic tests:

```toml
pulselink-transport = { version = "0.1", features = ["mock"] }
```

```rust
use pulselink_transport::mock::{MockBackend, MockLink};
use pulselink_transport::TransportDeviceId;

let link = MockLink::new(TransportDeviceId::new("device-1"));
let backend = MockBackend::new(Vec::new());
backend.push_link(link);
```

The mock supports notification injection, write capture, read data, failure
injection, disconnect, and reconnect queues.

## Dependency rule

`pulselink-runtime` depends on this crate, not on `pulselink-ble`. This keeps the
session runtime independent of platform BLE code and makes mock testing
straightforward.
