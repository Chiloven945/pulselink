# Hardware matrix

Real-device results are recorded here. CI cannot prove BLE behavior on physical hardware; run the
hardware tests manually or on a self-hosted runner.

## Procedure

```bash
PULSELINK_HW=1 cargo test -p pulselink-integration-tests --test hardware -- --ignored
```

The raw GATT test covers scan → connect → battery read → notify subscribe → zero soft-limit BF →
zero-intensity B0 → disconnect. The session test covers scan → connect → runtime initialization →
Ready/battery → shutdown.

## Results

| Date      | Platform | Adapter   | Device    | Firmware  | Result    | Notes                                        |
|-----------|----------|-----------|-----------|-----------|-----------|----------------------------------------------|
| _pending_ | Linux    | _pending_ | Coyote V3 | _pending_ | _not run_ | Record at least one real run before release. |
| _pending_ | Windows  | _pending_ | Coyote V3 | _pending_ | _not run_ |                                              |
| _pending_ | macOS    | _pending_ | Coyote V3 | _pending_ | _not run_ |                                              |

Do not mark a platform as verified based only on compilation. Record the adapter, device name,
firmware if available, and whether the zero-output checks passed.
