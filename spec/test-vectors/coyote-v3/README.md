# Coyote V3 test vectors

Canonical JSON vectors for Coyote V3 BLE.

Planned files:

- `b0.json` — B0 encode cases, including official HEX examples;
- `bf.json` — BF encode cases;
- `b1.json` — B1 decode cases;
- `battery.json` — battery decode cases;
- `frequency.json` — interval/wire conversion cases;
- `invalid.json` — length/header/range failures.

The Rust copy lives in `pulselink-rs/crates/pulselink-protocol/tests/vectors/coyote-v3/`.
