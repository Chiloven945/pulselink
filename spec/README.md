# pulselink protocol specification

This directory is the shared source of truth for protocol facts and cross-language test vectors used
by the Rust (`pulselink-rs`) and JVM (`pulselink4j`) implementations of `pulselink`.

## Contents

```text
protocols/
  coyote-v2/
  coyote-v3/
  civet/
  opossum/
  paw-prints/
  websocket-v4/
test-vectors/
  coyote-v2/
  coyote-v3/
  civet/
  opossum/
  paw-prints/
  websocket-v4/
```

## Implementations

| Implementation | Language                    | Status                                                  |
|----------------|-----------------------------|---------------------------------------------------------|
| `pulselink-rs` | Rust                        | `0.1.0` published; device and relay support in progress |
| `pulselink4j`  | Kotlin/JVM + Java extension | J0-J9 complete                                          |

Both implementations:

- treat this directory as canonical;
- keep protocol crates/modules sans-io;
- consume the same JSON test vectors;
- publish only their own language artifacts.

## Licensing

Do not copy the official DG-LAB documentation verbatim. The official protocol repositories state
that commercial use of the protocol content requires prior authorization. Files here are original
summaries, links, and test vectors derived from publicly documented wire behavior.

The Apache-2.0 license of the implementations covers source code only. It does not grant rights to
DG-LAB protocols, trademarks, or device firmware.

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

## Vector synchronization

Canonical vectors live here. Implementations may keep test-only copies because their build systems
cannot always include files outside the module directory.

Rust:

```bash
cd pulselink-rs
./scripts/sync-vectors.sh
git diff --exit-code
```

JVM:

- `pulselink-protocol` test source sets read `spec/test-vectors` during development;
- published jars do not include the specification;
- CI verifies the test-resource copy matches this directory.

Do not fabricate vectors for a protocol that has not been implemented and verified.
