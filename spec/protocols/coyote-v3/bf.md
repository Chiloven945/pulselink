# BF command

7 bytes, no response, effective immediately.

```text
byte 0: 0xBF
byte 1: channel A intensity soft limit
byte 2: channel B intensity soft limit
byte 3: channel A frequency balance
byte 4: channel B frequency balance
byte 5: channel A intensity balance
byte 6: channel B intensity balance
```

- soft limits: `0..=200`, persisted by the device;
- balance parameters: `0..=255`, persisted by the device;
- re-send BF after every reconnect;
- the SDK can only record that the transport accepted the write, not that the device confirmed it.
