# pulselink4j Implementation Roadmap

> Status: J0 build foundation, J1-J2 core/pattern/protocol, J3-J6 transport/BLE/device/runtime/
> WebSocket, and J8 shared documentation/spec are complete. J7 and J9 follow.
>
> Scope: JVM implementation of `pulselink`: Kotlin core plus an optional Java
> extension. Rust (`pulselink-rs`) remains the reference implementation for
> protocol behavior and test vectors.
>
> Target: `pulselink4j 0.1.0`.

---

## 1. Decisions

| Area          | Decision                                                     |
|---------------|--------------------------------------------------------------|
| Build         | Gradle **9.9.0**, Kotlin **2.4.21**                          |
| JVM bytecode  | Java **21**                                                  |
| CI JDK        | **21** and **25**                                            |
| Nullness      | Kotlin nullability; JSpecify **1.0.1** at the Java boundary  |
| BLE           | Kable **0.45.0** behind a `pulselink-transport` SPI          |
| WebSocket     | Ktor **3.6.1** client + CIO engine + WebSockets + coroutines |
| JSON          | kotlinx.serialization **1.12.0**                             |
| Coroutines    | kotlinx-coroutines **1.11.0**                                |
| Tests         | JUnit **6.1.3**, `kotlinx-coroutines-test`, MockK            |
| Docs          | Dokka **2.3.0**                                              |
| Java analysis | NullAway **0.14.3** (JSpecify mode)                          |

Gradle 9.9.0 is intentional. It is not Gradle 10, it supports Java 25, and it embeds Kotlin 2.4.21.
Keep the current wrapper. The Kotlin Gradle plugin compatibility table may lag behind Gradle
releases; CI validates the actual combination.

---

## 2. Java baseline and JSpecify

### Bytecode baseline

- Compile Kotlin and Java to **Java 21** bytecode.
- Run CI on Java 21 and Java 25.
- Use Java 21 language features only (records, sealed classes, pattern matching, text blocks); no
  preview features.
- Do not use Java 25-only APIs in the public API.

### JSpecify package rule

Every `pulselink-java` package must contain a `package-info.java` with:

```java
@NullMarked
package top.chiloven.pulselink.<module>.<package>;

        import org.jspecify.annotations.NullMarked;
```

Rules:

- `pulselink-java` has one `package-info.java` per Java package.
- Kotlin modules that expose Java-facing packages also include
  `src/main/java/<package>/package-info.java`.
- Kotlin source keeps Kotlin native nullability (`T` / `T?`) as the primary Kotlin contract.
- Java extension API uses `@Nullable` for nullable returns/parameters.
- Prefer `Optional<T>` only for optional value results; never return nullable collections or
  nullable `CompletableFuture`.
- Add `org.jspecify:jspecify:1.0.1` as an `api` dependency.

### Java analysis

`pulselink-java` runs NullAway in JSpecify mode:

```text
NullAway:OnlyNullMarked=true
NullAway:JSpecifyMode=true
```

JDK 21 requires `-XDaddTypeAnnotationsToSymbol=true` and an OpenJDK build (Temurin is fine). The
same checks must also run on JDK 25.

---

## 3. Module architecture

```text
pulselink4j/
├── pulselink-core          # domain types, no I/O
├── pulselink-pattern       # waveform timeline types
├── pulselink-protocol      # sans-io codecs and V4 types
├── pulselink-transport     # local-device SPI + test fake
├── pulselink-ble           # Kable adapter; hides Kable types
├── pulselink-device        # profiles and local device sessions
├── pulselink-runtime       # Coyote V3 runtime, ACK, scheduler, safety
├── pulselink-ws            # V4 Ktor WebSocket controller
├── pulselink-java          # optional Java 21 extension
├── pulselink-testing       # test fixtures, fake transport, vector readers
└── build-logic/            # Gradle convention plugins
```

Dependency direction:

```text
core
pattern -> core
protocol -> core + pattern
transport -> core
ble -> transport
device -> core + protocol + transport + runtime
runtime -> core + protocol + transport
ws -> core + protocol
java -> runtime + ws + optional ble
testing -> core + protocol + transport
```

`pulselink-ble` must not expose Kable types publicly. `pulselink-java` is a thin Java-facing adapter
and must not duplicate protocol/session logic.

Gradle has no Cargo-style features. Use separate artifacts instead:

- WebSocket-only users depend on `pulselink-ws`; they do not pull Kable/BLE.
- BLE users depend on `pulselink-runtime` + `pulselink-ble` + `pulselink-device`.
- Java users depend on `pulselink-java` (document that it aggregates runtime/WS, and optionally
  BLE).

---

## 4. Kotlin API style

- Public command methods are `suspend`.
- State is exposed as `StateFlow`.
- Events are exposed as `SharedFlow`.
- Domain objects are immutable `data class` / `sealed interface`.
- Constructors validate; do not silently clamp.
- Public errors use a sealed `PulseLinkException` hierarchy.
- Do not expose `kotlin.Result`, `GlobalScope`, or `runBlocking`.
- Use structured concurrency: one `SupervisorJob` per client/session.
- Per-link writes are serialized with a `Mutex` or a single-consumer channel.
- Value classes may be used internally; Java-facing APIs use plain types.

Example:

```kotlin
val link = PulseLinkClient()
val devices = link.scan(
    ScanFilter(namePrefixes = listOf("47L"), serviceUuids = emptyList())
)
val session = link.connectCoyoteV3(devices.first())

session.state.collect { state ->
    println("${state.connection} battery=${state.battery}")
}
session.setIntensity(Channel.A, Intensity.of(10))
session.play(pattern)
link.close()
```

Runtime design mirrors Rust:

- `SessionActor` with `Channel<Command>`.
- BLE notification flow into `SharedFlow`.
- Coyote V3 sequence 1..15, ACK queue, timeout/mismatch/zero output.
- 100 ms scheduler, pattern playback, safety policy.
- Reconnect with backoff; rewrite BF; do not restore pattern/intensity.
- Shutdown sends best-effort zero B0 and cancels the scope.

---

## 5. BLE with Kable

Dependency:

```kotlin
implementation("com.juul.kable:kable-core:0.45.0")
```

`pulselink-ble` maps Kable to the transport SPI:

- `Scanner` -> `Central.scan`.
- `Peripheral` -> `GattLink`.
- `Characteristic` -> UUID + properties.
- `Observation` -> `Flow<ByteArray>`.

Requirements:

- Apply the SDK `ScanFilter` as a second pass.
- Implement Coyote V3 privacy-broadcast fallback: name scan first, then service-UUID scan; classify
  unnamed `0x180C` devices as Coyote V3 candidates.
- Do not misclassify named Opossum/Civet/PawPrints devices.
- Query/negotiate MTU; skip Opossum B2 when MTU < 24.
- Serialize writes per link.
- Map platform failures to `TransportException`.
- Document Linux BlueZ permissions, macOS `NSBluetoothAlwaysUsageDescription`, and Windows Bluetooth
  permissions/radio state.

Fallback if Kable JVM Windows/macOS is insufficient:

1. Blue Falcon 3.x desktop engines; or
2. a JNI adapter over Rust `pulselink-ble`/btleplug implementing the same SPI.

Do not implement platform-specific BLE in the Java extension.

---

## 6. WebSocket with Ktor

Dependencies:

```kotlin
implementation("io.ktor:ktor-client-core:3.6.1")
implementation("io.ktor:ktor-client-cio:3.6.1")
implementation("io.ktor:ktor-client-websockets:3.6.1")
implementation("io.ktor:ktor-client-content-negotiation:3.6.1")
implementation("io.ktor:ktor-serialization-kotlinx-json:3.6.1")
```

Client:

```kotlin
val client = HttpClient(CIO) {
    install(WebSockets) {
        pingIntervalMillis = 30_000
        contentConverter = KotlinxWebsocketSerializationConverter(Json)
    }
}
```

Implementation requirements:

- Use Ktor CIO; it supports client WebSockets and is coroutine-native.
- Connect with `client.webSocketSession` and use `incoming`/`outgoing`
  channels.
- Send/receive text frames; JSON via kotlinx.serialization.
- Keep the Rust V4 behavior:
    - hello/targetId;
    - multiple app clients;
    - RPC reqId/pending/timeout;
    - devices snapshot/patch/slots.patch;
    - `device.op` submit/completion/clear;
    - heartbeat/ping/pong;
    - close codes 4000/4001/4002;
    - no silent reconnect and no reuse of an old targetId.
- Do not expose Ktor `Frame`/`HttpClient` types in the public API.

Tests use Ktor server:

```kotlin
testImplementation("io.ktor:ktor-server-core:3.6.1")
testImplementation("io.ktor:ktor-server-cio:3.6.1")
testImplementation("io.ktor:ktor-server-websockets:3.6.1")
```

The test server implements the same hello/client_attached/message flow and scripted RPC responses.
Cross-language integration may connect the JVM client to the Rust `pulselink-relay` binary.

---

## 7. Java extension

`pulselink-java` is Java 21 source, `java-library`, no Kotlin plugin.

API rules:

- `PulseLinkClient` and sessions implement `AutoCloseable`.
- Async methods return `CompletableFuture<T>` or `CompletionStage<T>`.
- Events use `java.util.concurrent.Flow.Publisher`.
- Nullable values use `@Nullable`; optional values use `Optional<T>`.
- No Kotlin coroutines, `Flow`, default arguments, or value classes in public Java signatures.
- Every `pulselink-java` package has `package-info.java` with `@NullMarked`.
- Run NullAway in JSpecify mode on `pulselink-java`.

Example:

```java
try(var link = PulseLinkClient.builder().build()){
var devices = link.scan(ScanFilter.all()).get();
var session = link.connectCoyoteV3(devices.get(0)).get();
    session.

setIntensity(Channel.A, 10).

get();
}
```

The Java extension delegates to Kotlin runtime/WS objects and contains no protocol logic.

---

## 8. Shared monorepo spec and docs

### Keep in `pulselink-rs/docs`

Rust-specific documents:

- `ROADMAP.md`
- `runtime-contracts.md`
- `release.md`
- `adr/`
- Rust crate-specific architecture notes

### Move language-neutral docs to root `docs/`

```text
docs/
├── architecture.md
├── devices.md
├── hardware-matrix.md
├── platforms.md
├── security.md
├── testing.md
├── local-relay.md
└── jvm.md
```

### Expand `spec/`

Add missing protocol pages and vectors:

```text
spec/protocols/
├── coyote-v2/
├── coyote-v3/
├── civet/
├── opossum/
├── paw-prints/
└── websocket-v4/
```

```text
spec/test-vectors/
├── coyote-v2/
├── coyote-v3/
├── civet/
├── opossum/
├── paw-prints/
└── websocket-v4/
```

Rules:

- `spec/` is the canonical source of protocol facts and vectors.
- Rust and JVM both consume the same vectors.
- JVM tests read vectors from the source tree; published jars do not include the spec.
- CI checks Rust crate copies and JVM test resources against `spec/`.
- Keep the commercial-use notice in `spec/README.md`.

`pulselink4j/docs/` contains JVM-only documents:

- Gradle/Kotlin/JDK compatibility.
- Java extension nullness rules.
- BLE native dependency and permission notes.
- Maven Central publishing.

---

## 9. Gradle build architecture

- Root project applies only `base`; no root library.
- `build-logic` included build provides convention plugins:
    - `pulselink.kotlin-library`
    - `pulselink.java-library`
    - `pulselink.publish`
    - `pulselink.test`
- `settings.gradle.kts` defines `pluginManagement` and
  `dependencyResolutionManagement`.
- `gradle/libs.versions.toml` owns all versions.
- Kotlin libraries use `explicitApi()`, `allWarningsAsErrors`, and
  `binary-compatibility-validator`.
- Java extension uses `--release 21`, `-Xlint:all`, and `-Werror`.
- Every published module produces sources and Dokka/Javadoc artifacts.
- Maven Central publishing uses `maven-publish` + signing (or the Vanniktech plugin) and complete
  POM metadata.

Version catalog highlights:

```toml
[versions]
kotlin = "2.4.21"
coroutines = "1.11.0"
serialization = "1.12.0"
jspecify = "1.0.1"
kable = "0.45.0"
ktor = "3.6.1"
junit = "6.1.3"
dokka = "2.3.0"
nullaway = "0.14.3"
slf4j = "2.0.17"
```

---

## 10. Milestones

### J0 — Build foundation (complete)

- Rewrite root build/settings.
- Add version catalog and build-logic convention plugins.
- Configure Kotlin 2.4.21, Gradle 9.9.0, Java 21 toolchain.
- Add JSpecify, JUnit 6, Dokka, API validation, publishing.
- Add `.gitignore` for `.gradle/`, `build/`, `.idea/`.

Acceptance: `./gradlew projects`, `./gradlew check` on empty modules.

Completion notes: Gradle 9.9.0 + Kotlin 2.4.21, convention plugins, version catalog, BCV API dumps,
JSpecify package-info in `pulselink-java`, and a green `./gradlew check`.

### J1 — Core/pattern (complete)

- `DeviceId`, `DeviceKind`, `Channel`, `Intensity`, `Pressure`,
  `ButtonMask`, `Trigger`.
- `DeviceState`, `Observed`, `DeviceCapabilities`, `DeviceEvent`.
- `pulselink-pattern`: pulse samples, channel frames, non-empty timelines.
- Kotlin tests for bounds and state revisions.

### J2 — Protocol (complete)

- Coyote V2/V3, Civet, Opossum, PawPrints codecs.
- V4 relay frames, RPC, device model, events, operations.
- kotlinx.serialization configuration.
- Shared test-vector reader and tests.

**J1 completion notes**: core and pattern are implemented with validated value types and unit tests;
Kotlin nullability is used directly.

**J2 completion notes**: Coyote V2/V3, Civet, Opossum, PawPrints, and V4 codecs are implemented. V4
unknown frames/events preserve raw JSON; operations validate cross-field constraints. V3/V4 tests
consume the shared `spec/test-vectors` resources; V2/sensor tests use documented wire cases.
Dedicated V2/sensor vector JSON remains a follow-up.

### J3 — Transport

- `Central`, `GattLink`, `Connector`, `Backend`.
- `Flow<ByteArray>` notifications, `WriteMode`, MTU.
- Fake transport and test fixtures.

**J3 completion notes**: the transport SPI is implemented with coroutine `Flow` notifications,
`GattCharacteristic`, `CharacteristicInfo`, `TransportException`, and a protocol-agnostic `Backend`.
`pulselink-testing` provides `MockGattLink` and `MockBackend`, including notification injection,
write capture, read data, failure injection, MTU, and connection state.

### J4 — BLE

- Kable adapter and profile classifier.
- Scan second pass and V3 privacy fallback.
- Notification fan-out, write serialization, reconnect.
- Platform permission docs.

**J4 completion notes**: `KableBackend`/`KableGattLink` implement the transport SPI with Kable
0.45.0. Kable's JVM target uses btleplug FFI/JNI, so it shares the same desktop platform family as
the Rust backend. Scans use Kable filters plus a second-pass `ScanFilter`,
connection/discovery/write/read/disconnect have timeouts, writes are serialized, and reconnect falls
back to a filtered scan. Kable JVM currently reports an ATT MTU of 23, so Opossum `0xB2` display
sync must be skipped on JVM unless a platform-specific MTU path is added.

### J5 — Device/runtime

- Device profiles and local sessions.
- Coyote V3 actor, ACK, scheduler, pattern player, safety, reconnect.
- Other device sessions.
- Mock integration tests.

**J5 completion notes**: `pulselink-runtime` provides `Client` and `CoyoteV3Session` with B0/BF
encoding, B1 ACK tracking, 100 ms pattern playback, safety limits, reconnect, and shutdown zeroing.
`pulselink-device` provides `DeviceClient`, name-first/service-fallback profiles, and sessions for
Coyote V2/V3, Opossum, Civet, and PawPrints. The JVM currently reports ATT MTU 23, so Opossum B2 is
skipped.

### J6 — WebSocket

- Ktor CIO client, WebSocket plugin, serialization.
- Socket/AppClient/RPC/registry/cache/operations.
- Typed V4 snapshots and pulse v2.
- Ktor test server integration tests.

**J6 completion notes**: `pulselink-ws` uses Ktor CIO + WebSockets + kotlinx.serialization JSON.
`Socket`, `AppClient`, `RemoteDeviceHandle`, `OperationHandle`, device cache, multi-app registry,
RPC timeouts, clear operations, heartbeat/pong, and close handling are implemented. Ktor embedded
server tests cover hello, client attach, devices.get, ping, clear, and shutdown.

### J7 — Java extension

- Java 21 `PulseLinkClient` and sessions.
- `CompletableFuture`, `Flow.Publisher`, `AutoCloseable`.
- `package-info.java` with `@NullMarked` for every `pulselink-java` package.
- NullAway JSpecify checks on JDK 21/25.

### J8 — Docs/spec (complete)

- Move language-neutral docs to root.
- Expand `spec/protocols` and `spec/test-vectors`.
- Add JVM docs and update Rust links.
- Add monorepo implementation matrix to `spec/README.md`.

Completion notes: moved device/platform/security/testing/local-relay/V3 privacy docs to root
`docs/`; added root README, docs index, monorepo architecture, JVM build/nullness docs, protocol
summaries for Coyote V2/Civet/Opossum/PawPrints, and placeholder vector directories. Local Markdown
links were checked; no guessed vectors were added.

### J9 — CI/publish

- Linux/Windows/macOS matrix.
- JDK 21/25 matrix.
- API check, Dokka, test vectors.
- `publishToMavenLocal` and Maven Central dry run.
- CREDENTIALS/signing documentation.

---

## 11. Verification

Local:

```text
./gradlew check
./gradlew apiCheck
./gradlew dokkaGenerateHtml
./gradlew publishToMavenLocal
```

CI:

```text
Linux  JDK 21
Linux  JDK 25
Windows JDK 21
macOS  JDK 21
```

Hardware/V4:

- BLE hardware tests gated by `PULSELINK_HW=1`.
- V4 tests against Ktor test server and optionally the Rust local relay.
- Record real results in `../docs/hardware-matrix.md`.

---

## 12. Risks

- Kable JVM Windows/macOS behavior needs real hardware validation.
- Ktor CIO WebSocket behavior and close-code handling must be covered by tests.
- JSpecify `package-info.java` must be added for every `pulselink-java` package.
- NullAway JSpecify mode on JDK 21 needs the type-annotation compiler flag.
- Gradle 9.9.0 is newer than the KGP compatibility table; CI is the source of truth.
- BLE native dependencies must stay isolated in `pulselink-ble`; do not leak them into
  `pulselink-ws`.
- Moving docs/spec must update all Rust and JVM links in one change.
