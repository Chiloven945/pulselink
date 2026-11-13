# pulselink4j

JVM implementation of `pulselink`: a Kotlin core with an optional Java extension.

Status: **J0 build foundation through J9 CI/publish are complete**.
See [`ROADMAP.md`](ROADMAP.md).

## Toolchain

| Component     | Version                      |
|---------------|------------------------------|
| Gradle        | 9.9.0                        |
| Kotlin        | 2.4.21                       |
| Java bytecode | 21                           |
| CI JDK        | 21 and 25                    |
| Java compile  | toolchain configurable (25 default), `--release 21` |
| JSpecify      | 1.0.1                        |
| BLE           | Kable 0.45.0                 |
| WebSocket     | Ktor 3.6.1 + CIO             |

## Modules

| Module                | Purpose                                          |
|-----------------------|--------------------------------------------------|
| `pulselink-core`      | Domain types, state, capabilities, events        |
| `pulselink-pattern`   | Waveform timeline types                          |
| `pulselink-protocol`  | Sans-io Coyote/Civet/Opossum/PawPrints/V4 codecs |
| `pulselink-transport` | Local-device SPI and test fake                   |
| `pulselink-ble`       | Kable adapter; hides Kable types                 |
| `pulselink-device`    | Profiles and local device sessions               |
| `pulselink-runtime`   | Coyote V3 session, ACK, scheduler, safety        |
| `pulselink-ws`        | DG-LAB V4 WebSocket controller                   |
| `pulselink-java`      | Java 21 extension                                |
| `pulselink-testing`   | Shared test fixtures                             |

## Build

```bash
./gradlew check
./gradlew apiCheck
./gradlew dokkaGenerateHtml
./gradlew publishToMavenLocal
```

Publishing to Maven Central uses `nmcp` aggregation:

```bash
./gradlew publishAggregationToCentralPortal -PpulselinkVersion=0.1.0
```

Credentials, signing, and the release workflow are documented in
[`docs/releasing.md`](docs/releasing.md).

`apiDump` must be run after intentional public API changes:

```bash
./gradlew apiDump
```

## Documentation

- [`ROADMAP.md`](ROADMAP.md) — implementation plan and milestones
- [`docs/build.md`](docs/build.md) — Gradle/build conventions
- [`docs/releasing.md`](docs/releasing.md) — Maven Central credentials, signing, and release workflow
- [`docs/nullness.md`](docs/nullness.md) — JSpecify and Java nullness rules
- [`docs/java.md`](docs/java.md) — Java 21 extension quick start and async model
- [`../docs/`](../docs/) — shared device/platform/security documentation
- [`../spec/`](../spec/) — shared protocol facts and test vectors

## Protocol notice

The DG-LAB protocol documentation states that commercial use of the protocol content requires prior
authorization from DG-LAB. The Apache-2.0 license covers this repository's code only.
