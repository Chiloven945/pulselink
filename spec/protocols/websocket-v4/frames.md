# V4 outer frames

| Type                      | Direction           | Purpose                         |
|---------------------------|---------------------|---------------------------------|
| `hello`                   | server → client     | Assign `clientId`               |
| `client_attached`         | server → controller | App attached                    |
| `client_disconnected`     | server → controller | App disconnected                |
| `controller_attached`     | server → app        | App attached to controller      |
| `controller_disconnected` | server → app        | Controller disconnected         |
| `heartbeat`               | server → client     | Every 30 s                      |
| `pong`                    | server → client     | Response to server-level `ping` |
| `message`                 | both                | Application message             |
| `idle_timeout`            | server → controller | No app for 5 minutes            |
| `error`                   | server → client     | Error                           |

Controller message:

```json
{
  "type": "message",
  "clientId": "<app-client-id>",
  "data": {}
}
```

Close codes:

- `4000` — controller disconnected;
- `4001` — controller not found;
- `4002` — idle timeout.
