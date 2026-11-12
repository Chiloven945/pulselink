# JVM BLE backend

`pulselink-ble` implements the `pulselink-transport` SPI with Kable 0.45.0. Kable's JVM target
currently drives btleplug through FFI/JNI, so its supported desktop platforms mirror the Rust
backend: Linux/BlueZ, Windows/WinRT, and macOS/CoreBluetooth.

## Usage

```kotlin
val backend = KableBackend()
val devices = backend.scan(
    ScanFilter(namePrefixes = listOf("47L")),
    timeout = 5.seconds,
)
val link = backend.connect(devices.first())
link.write(
    GattCharacteristic(SERVICE_CONTROL, CHAR_WRITE),
    bytes,
    WriteMode.Auto,
)
link.disconnect()
backend.close()
```

`KableBackend` is `AutoCloseable`; close it when the BLE session is finished.
`GattLink.subscribe` returns a cold `Flow<ByteArray>`; each collector owns its own notification
subscription.

## Platform requirements

### Linux

- BlueZ must be running.
- The user needs access to the system Bluetooth adapter (`bluetooth` group or equivalent D-Bus
  policy).
- Kable's JVM artifact loads btleplug's native library; validate packaging for the target
  distribution.

### macOS

- Add `NSBluetoothAlwaysUsageDescription` to the application `Info.plist`.
- Validate the native library inside the application bundle or distribution.

### Windows

- Bluetooth must be enabled and the application must have permission to use it.
- Kable/btleplug uses the WinRT BLE stack through FFI/JNI.
- Validate native-library loading in the packaged application.

## MTU

Kable's JVM `maximumWriteValueLengthForType` currently reports an ATT payload of 20 bytes (ATT MTU
23). `KableGattLink.mtu()` therefore returns 23. Opossum `0xB2`
display sync requires at least 24 bytes and must be skipped on JVM until a platform-specific MTU
path is added.

## Reconnect

`KableBackend.reconnect` first tries the cached peripheral. If that fails, it scans using the
supplied `ScanFilter` and connects to the matching device. Like the Rust backend, it does not
restore device output or protocol settings; that is the responsibility of the device/runtime layer.

## Hardware validation

Unit tests use fakes. Real-device validation must be recorded in
[`../../docs/hardware-matrix.md`](../../docs/hardware-matrix.md) and should cover:

- scan and second-pass filtering;
- connect and service discovery;
- characteristic read/write/notify;
- write-without-response vs write-with-response;
- disconnect/reconnect;
- native-library packaging on each platform.
