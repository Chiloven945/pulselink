# Platforms

`pulselink` 0.1 targets desktop platforms first.

## Linux

- BlueZ must be available; the user needs permission to access the Bluetooth adapter.
- `pulselink-ble` builds a vendored libdbus on Linux, so no system D-Bus development headers are
  required.
- btleplug scan filtering is not fully isolated by service UUID; `pulselink` performs a second-pass
  filter in the SDK.
- Multiple adapters are possible. `BtleplugCentral::new()` (Rust) and the Kable-backed JVM
  implementation must handle an empty adapter list without panicking.
- Kable's JVM target uses btleplug FFI/JNI. The native library must be present in the application
  runtime; packaging validation is required per platform.

## macOS

- The application needs Bluetooth usage descriptions and entitlements.
- Peripheral identifiers differ from Linux/Windows and are not guaranteed to be stable across
  restarts.
- Notification behavior should be verified on a real device.
- The hardware probe is disabled unless `PULSELINK_HW=1` is set; it defaults to battery/notify and
  zero-intensity output only.

## Windows

- WinRT Bluetooth permissions and the Bluetooth radio state matter.
- Kable's JVM target uses btleplug's WinRT backend through FFI/JNI; validate native-library
  packaging with the application distribution.
- Characteristic properties may differ; always check `WriteWithoutResponse`
  support instead of hardcoding the write mode.
- Real-device testing is recommended before claiming support.

## Android/iOS

Not first-class in 0.1. The `pulselink-transport` port is designed so a platform backend can be
added without changing the protocol or runtime layers.
