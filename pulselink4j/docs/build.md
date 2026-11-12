# JVM build

## Toolchain

- Gradle 9.9.0 (wrapper)
- Kotlin 2.4.21
- Java toolchain 21
- Java extension source/target 21
- CI runs JDK 21 and JDK 25

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
- `pulselink.java-library` — Java 21 library with JSpecify and strict lint. The Javadoc jar is
  deferred until J7 adds public Java classes.
- `pulselink.test` — JUnit 6 BOM and coroutine test support.
- `pulselink.publish` — Maven publication metadata and signing.
- `pulselink.serialization` — kotlinx.serialization compiler plugin.

## Common commands

```bash
./gradlew projects
./gradlew check
./gradlew apiCheck
./gradlew apiDump
./gradlew dokkaGenerateHtml
./gradlew publishToMavenLocal
```

`pulselink-testing`, `pulselink-lab`-style tools, and integration-test artifacts are not published.
