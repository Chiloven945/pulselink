# pulselink-core

Core domain types for pulselink-rs. This crate has no I/O and no async runtime.

It defines:

- `DeviceId`, `BleDeviceId`, `RemoteDeviceId`, `DeviceKind`, `DeviceInfo`;
- `Channel`, `ChannelMap`;
- `Intensity`, `IntensityDelta`, `BatteryLevel`;
- `Observed<T>`, `ConnectionState`, `DeviceState`, `ChannelState`, `BatteryState`;
- `DeviceCapabilities`, `DeviceEvent`, `CoreError`.

The most important design rule is `Observed<T>`: a freshly connected device has unknown intensity,
not zero intensity.

```rust
use pulselink_core::{Channel, ChannelMap, Intensity};

let intensity = Intensity::new(10) ?;
let limits = ChannelMap::new(intensity, Intensity::MIN);
assert_eq!(limits[Channel::A], intensity);
# Ok::<(), pulselink_core::CoreError>(())
```

- Protocol bytes: `pulselink-protocol`.
- Device sessions: `pulselink-runtime`.
- Full SDK: `pulselink`.

See `docs/ROADMAP.md` for the project plan.
