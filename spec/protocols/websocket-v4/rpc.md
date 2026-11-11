# V4 RPC

Inner messages use `t`:

```json
{
  "t": "req",
  "reqId": "req-1",
  "m": "devices.get",
  "data": {}
}
{
  "t": "resp",
  "reqId": "req-1",
  "result": {}
}
{
  "t": "resp",
  "reqId": "req-1",
  "error": "slot_not_found"
}
{
  "t": "ev",
  "ev": "devices.snapshot",
  "devices": []
}
```

- `reqId` must be unique among pending requests for the same app client.
- `result` and `error` are mutually exclusive.
- `device.op` uses `reqId` as the task ID on the app side.
- Default response timeout is 8000 ms in the official SDK; `pulselink` makes it configurable.
