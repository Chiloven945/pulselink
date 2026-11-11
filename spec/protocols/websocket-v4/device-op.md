# V4 device operations

`device.op` data:

| Field | Meaning                                     |
|-------|---------------------------------------------|
| `s`   | slotId                                      |
| `t`   | ActionType                                  |
| `c`   | `0` = A, `1` = B                            |
| `p`   | priority `0..=2`, default 1                 |
| `d`   | duration in ms, default 0                   |
| `v`   | intensity value or pulse frames             |
| `im`  | replace tasks with same device/channel/type |
| `ver` | pulse frame version 2 or 3                  |
| `seq` | legacy frame sequence                       |

Action types:

| `t` | Type             | Lifecycle                                      |
|----:|------------------|------------------------------------------------|
|   0 | AppendPulseData  | Continuous; completes when frames are consumed |
|   3 | AddIntensity     | One-shot; `v` may be negative                  |
|   4 | SetTempIntensity | Continuous; resets to 0 when the task ends     |
|   7 | SetIntensity     | One-shot; only `v = 0`                         |

Response:

```json
{
  "type": 7,
  "reason": "completed",
  "slotId": "slot-a",
  "channel": 1
}
```

`reason`: `completed`, `cleared`, `replaced`, `cancelled`.

`device.op.clear`:

- no data: clear all tasks;
- `{ "s": "slot-a" }`: clear all channels of one device;
- `{ "s": "slot-a", "c": 0 }`: clear one channel;
- `c` without `s` is invalid;
- returns `{}`; each cleared task also receives its own response with `reason: "cleared"`.
