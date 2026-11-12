# pulselink-lab

Interactive terminal lab for manually testing `pulselink` devices.

The lab is intentionally a separate, non-published crate. It uses the facade APIs plus the
non-published local relay; it is not part of the published SDK.

## Run

```bash
cargo run -p pulselink-lab
```

The top row shows the selected backend, output gate, and connection state. Press `F1` for the
built-in key map.

## Safety

Device output is disabled by default. Press `Space` to enable it. The command
`stopall` and `F12` always attempt an emergency zero/stop on the current session.

## Commands

Press `:` to enter command mode.

Coyote V3 firmware updated by the DG-LAB 4.x app may hide `47L121000` from
advertising. The lab shows such units as `(unnamed)` when they still advertise
service `0x180C`; connect by index, or switch the device to public broadcast.
See [`docs/coyote-v3-privacy-broadcast.md`](../../../docs/coyote-v3-privacy-broadcast.md).

```text
scan [v2|v3|paw|civet|opossum]
connect <index>
disconnect
output on|off
max <0..200>
int <a|b> <0..200>
temp <a|b> <0..200> <ms>
play
stop
reconnect
v2 int <a|b> <0..2047>
v2 pulse <a|b> <x> <y> <z>
op int <a|b> <0..200>
op pulse <a|b> <v1> <v2> <v3> <v4>
civet start|stop|reset
civet orientation <1|3>
paw notrigger|reset|angles|light <0..255>
ws connect <url>
ws local
relay start [addr]
relay stop
relay status
ws refresh
ws int <slot> <a|b> <0..200>
ws reset <slot> <a|b>
ws clear
stopall
quit
```

`ws local` starts a real local WebSocket relay on `0.0.0.0:8787`, connects the controller, and
prints the target id plus a pairing URL. Open the pairing URL (or its QR URL) in the official
DG-LAB app on the same LAN. `relay start` starts the relay without connecting a controller.

The relay uses plain `ws://` for local development. Some app versions may require `wss://` or a
trusted certificate; in that case put the relay behind a TLS tunnel/reverse proxy.

The terminal UI uses crossterm/ratatui so it works over SSH and in headless CI without X11 or
Wayland.
