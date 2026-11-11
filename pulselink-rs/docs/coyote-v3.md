# Coyote V3 BLE protocol

This is a working summary for `pulselink`. It is not a copy of the official documentation. Always
verify against the official DG-LAB Bluetooth protocol repository.

## GATT

| Service  | Characteristic | Properties  | Purpose   |
|----------|----------------|-------------|-----------|
| `0x180C` | `0x150A`       | Write       | Commands  |
| `0x180C` | `0x150B`       | Notify      | Responses |
| `0x180A` | `0x1500`       | Read/Notify | Battery   |

Base UUID: `0000xxxx-0000-1000-8000-00805f9b34fb`.

Device name for Stimulator 3.0: `47L121000`.

## B0

20 bytes, sent every 100 ms, both channels:

```text
0xB0
+ sequence(4 bits) + modeA(2 bits) + modeB(2 bits)
+ intensityA + intensityB
+ freqA[4] + ampA[4]
+ freqB[4] + ampB[4]
```

- Intensity: `0..=200`.
- Mode: keep / increase / decrease / set.
- Pulse frequency wire value: `10..=240`.
- Pulse intensity (amplitude): `0..=100`.
- Four 25 ms pairs per channel cover 100 ms.
- An amplitude above 100 makes the device discard that channel's four pairs; `pulselink` models this
  as `ChannelWaveform::Silent`.

## BF

7 bytes, no response, effective immediately:

```text
0xBF + softLimitA + softLimitB
    + freqBalanceA + freqBalanceB
    + intensityBalanceA + intensityBalanceB
```

Re-send BF after every reconnect. `pulselink` records "write accepted by the transport", not "device
confirmed".

## B1

4 bytes:

```text
0xB1 + sequence + actualA + actualB
```

- Sequence 0 means an unsolicited device-side change.
- Sequence 1..=15 matches a B0 intensity request.
- A mismatched B1 still updates observed intensity but must not complete the pending request.

## Pulse interval conversion

The official "pulse frequency value" is in milliseconds:

```text
input 10..=1000 ms
10..=100   -> input
101..=600  -> (input - 100) / 5 + 100
601..=1000 -> (input - 600) / 10 + 200
wire 10..=240
```

`10 ms` is `100 Hz`; `1000 ms` is `1 Hz`.

## Safety

- The protocol maximum is not a human-safety recommendation.
- BF soft limits and the SDK `SafetyPolicy` are independent layers.
- If the BLE link is down, software cannot guarantee that a stop command reaches the device.
  Device-side protection must not be disabled.

## Reference

<https://github.com/dungeonlab-open/dglab-bluetooth-protocol>

The official protocol documentation states that commercial use requires prior authorization from
DG-LAB.
