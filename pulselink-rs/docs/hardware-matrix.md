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
| _pending_ | Linux    | _pending_ | Coyote V2 | _pending_ | _not run_ | PWM_AB2/A34/B34; zero output only first.     |
| _pending_ | Linux    | _pending_ | PawPrints | _pending_ | _not run_ | No-trigger 0x50 on connect; event stream.    |
| _pending_ | Linux    | _pending_ | Civet     | _pending_ | _not run_ | Pressure start/stop and reset.               |
| _pending_ | Linux    | _pending_ | Opossum   | _pending_ | _not run_ | B0/B3/B2; record MTU.                        |

Do not mark a platform as verified based only on compilation. Record the adapter, device name,
firmware if available, and whether the zero-output checks passed.

Coyote V3 units updated by the DG-LAB 4.x app may use privacy broadcast and omit
`47L121000` from advertising. Switch the device to public broadcast before a name-based hardware
run, or record that the service-UUID fallback was used. See
[`coyote-v3-privacy-broadcast.md`](coyote-v3-privacy-broadcast.md).
