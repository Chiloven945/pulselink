# JVM implementation

`pulselink4j` is the Kotlin/JVM implementation of `pulselink`.

## Toolchain

- Gradle 9.9.0
- Kotlin 2.4.21
- Java 21 bytecode; CI on JDK 21 and 25
- Kotlin nullability; JSpecify 1.0.1 `@NullMarked` packages in `pulselink-java`
- Kable 0.45.0 for BLE
- Ktor 3.6.1 (CIO + WebSockets) for DG-LAB V4

## Modules

- `pulselink-core`
- `pulselink-pattern`
- `pulselink-protocol`
- `pulselink-transport`
- `pulselink-ble`
- `pulselink-device`
- `pulselink-runtime`
- `pulselink-ws`
- `pulselink-java` (optional Java 21 extension)
- `pulselink-testing` (non-published)

## Links

- [`pulselink4j/README.md`](../pulselink4j/README.md)
- [`pulselink4j/ROADMAP.md`](../pulselink4j/ROADMAP.md)
- [`pulselink4j/docs/build.md`](../pulselink4j/docs/build.md)
- [`pulselink4j/docs/nullness.md`](../pulselink4j/docs/nullness.md)

Shared device/platform/security documentation is in [`docs/`](README.md). Protocol facts and vectors
are in [`spec/`](../spec/).
