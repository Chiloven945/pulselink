# Waveform semantics

- Each B0 contains four frequency/intensity pairs per channel.
- Each pair covers 25 ms; four pairs cover 100 ms.
- The official "pulse frequency value" is in milliseconds:
    - `10 ms` = `100 Hz`;
    - `1000 ms` = `1 Hz`.
- The V3 wire value is compressed to `10..=240`:

```text
input 10..=100   -> input
input 101..=600  -> (input - 100) / 5 + 100
input 601..=1000 -> (input - 600) / 10 + 200
```

- Pulse intensity (amplitude) is `0..=100`.
- A value above 100 causes the device to discard all four pairs for that channel. `pulselink` models
  this as explicit silence.
- The reverse mapping from wire value to milliseconds is not unique; it is only suitable for
  display.
