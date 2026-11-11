# pulselink protocol specification

This directory is the shared source of truth for protocol facts and cross-language test vectors used
by the Rust (`pulselink-rs`) and JVM (`pulselink4j`)
implementations of `pulselink`.

## Contents

```text
protocols/
  coyote-v3/
  websocket-v4/
test-vectors/
  coyote-v3/
  websocket-v4/
```

## Licensing

Do not copy the official DG-LAB documentation verbatim. The official protocol repositories state
that commercial use of the protocol content requires prior authorization. Files here are original
summaries, links, and test vectors derived from publicly documented wire behavior.

## Test vector schema

```json
{
  "version": 1,
  "cases": [
    {
      "id": "b0-absolute-a-10",
      "kind": "b0.encode",
      "input": {},
      "expect": {
        "hex": "B0..."
      },
      "notes": "optional"
    }
  ]
}
```

## Sync with Rust

Cargo packages cannot include files outside their crate directory. Canonical vectors live here;
`pulselink-protocol` keeps a copy under `pulselink-rs/crates/pulselink-protocol/tests/vectors/`. CI
must run the sync script and fail if the copies differ.

```bash
cd pulselink-rs
./scripts/sync-vectors.sh
git diff --exit-code
```
