# JVM build

## Toolchain

- Gradle 9.9.0 (wrapper)
- Kotlin 2.4.21
- Java toolchain: configurable, default 25
- Kotlin/Java bytecode target: 21
- CI runs JDK 21 and JDK 25
- JDK 21 must be 21.0.8+ for NullAway JSpecify mode

The toolchain is selected with:

```bash
./gradlew check -PjavaToolchainVersion=21
```

`pulselink.kotlin-library` and `pulselink.java-library` both honor this property. `jvmTarget` and
`options.release` stay at 21, so the published bytecode is Java 21 even when the build runs on
JDK 25.

Gradle 9.9.0 is intentional. It is not Gradle 10, supports Java 25, and embeds Kotlin 2.4.21.

## Structure

```text
pulselink4j/
├── build-logic/          # convention plugins
├── gradle/libs.versions.toml
├── pulselink-core/
├── pulselink-pattern/
├── pulselink-protocol/
├── pulselink-transport/
├── pulselink-ble/
├── pulselink-device/
├── pulselink-runtime/
├── pulselink-ws/
├── pulselink-java/
└── pulselink-testing/
```

The root project is a pure aggregator (`base` plugin). It does not publish an artifact.

## Convention plugins

- `pulselink.kotlin-library` — Kotlin/JVM library, Dokka, BCV, explicit API, JUnit 6.
- `pulselink.java-library` — Java 21 library with JSpecify and strict lint.
- `pulselink.test` — JUnit 6 BOM and coroutine test support.
- `pulselink.publish` — Maven publication metadata, PGP signing, and nmcp variant.
- `pulselink.serialization` — kotlinx.serialization compiler plugin.

## Common commands

```bash
./gradlew projects
./gradlew check
./gradlew apiCheck
./gradlew apiDump
./gradlew dokkaGenerateHtml
./gradlew publishToMavenLocal
./gradlew publishAggregationToCentralPortal
```

The last task requires Central Portal credentials and signing secrets. See [
`releasing.md`](releasing.md).

`pulselink-testing`, `pulselink-lab`-style tools, and integration-test artifacts are not published.
