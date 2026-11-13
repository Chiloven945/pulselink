# JVM implementation

`pulselink4j` is the Kotlin/JVM implementation of `pulselink`.

## Toolchain

- Gradle 9.9.0
- Kotlin 2.4.21
- Java 21 bytecode; CI on JDK 21 and 25
- Kotlin nullability; JSpecify 1.0.1 `@NullMarked` packages in `pulselink-java`
- Kable 0.45.0 for BLE; its JVM target uses btleplug FFI/JNI
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
- `pulselink-java` (optional Java 21 extension: Java ports + Kotlin adapters)
- `pulselink-testing` (non-published)

Transport/BLE currently expose `Central`, `GattLink`, `Connector`, and `Backend` with coroutine
`Flow<ByteArray>` notifications. Kable JVM reports an ATT MTU of 23, so Opossum B2 display sync must
be skipped on JVM. Native Kable libraries and real-hardware validation are still required on each
platform.

The Java extension exposes Java interfaces, records, and sealed event types. All asynchronous
operations return `CompletableFuture`, state/event streams use JDK `Flow.Publisher`, and resource
owners implement `AutoCloseable`. Java public packages are `@NullMarked`; Kotlin adapters live in an
internal package and are not part of the public API.

## Links

- [`pulselink4j/README.md`](../pulselink4j/README.md)
- [`pulselink4j/ROADMAP.md`](../pulselink4j/ROADMAP.md)
- [`pulselink4j/docs/build.md`](../pulselink4j/docs/build.md)
- [`pulselink4j/docs/nullness.md`](../pulselink4j/docs/nullness.md)

Shared device/platform/security documentation is in [`docs/`](README.md). Protocol facts and vectors
are in [`spec/`](../spec/).

- [`pulselink4j/docs/ble.md`](../pulselink4j/docs/ble.md)
