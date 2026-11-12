# Coyote V3 privacy broadcast

Recent Coyote V3 firmware, updated through the DG-LAB 4.x app, can enter a privacy-broadcast mode.
The device continues to advertise and keeps the V3 GATT services, but it no longer includes the
`47L121000` complete local name. A name-prefix scan such as `47L121000` therefore misses it.

This is documented upstream:

- [dglab-bluetooth-protocol issue #48](https://github.com/dungeonlab-open/dglab-bluetooth-protocol/issues/48)
- [dglab-bluetooth-protocol issue #50](https://github.com/dungeonlab-open/dglab-bluetooth-protocol/issues/50)

The official reply confirms that there is no separate V4 BLE protocol; the V3 GATT service and
B0/BF/B1 commands remain valid. A device bound to the 4.x app must be switched to public broadcast
before third-party tools can find it by name.

## Switch to public broadcast

1. Power on the Coyote V3.
2. Push both side wheels **down** together (not press the wheels).
3. Hold until the main indicator light pattern changes.
4. Scan again; the device should advertise `47L121000`.

If the operating system cached the old advertisement, remove the device first:

```bash
bluetoothctl remove <MAC>
bluetoothctl scan on
```

To confirm privacy mode, inspect the advertisement:

```bash
sudo btmon
```

If the device address has no `Complete Local Name: 47L121000` field, it is using privacy broadcast.

## pulselink behavior

`DeviceProfile::scan_filter()` keeps the fast name-first scan.
`DeviceProfile::fallback_scan_filter()` retries with the profile service UUID when the name scan
finds nothing. `DeviceClient::scan_profile()` runs the fallback automatically.

An unnamed device advertising `0x180C` is treated as a **Coyote V3 candidate**. This is a pragmatic
fallback because V3 is the currently documented device with privacy broadcast; `0x180C` is shared
with Opossum, Civet, and PawPrints. The lab labels it `(unnamed)` and prints a warning. If
connecting and V3 initialization fails, switch the device to public broadcast and rescan.

The robustness fallback does not replace the official procedure: in privacy mode the device does not
expose a name, so third-party discovery cannot reliably identify it by name alone.
