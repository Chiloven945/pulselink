# JSpecify nullness

`pulselink4j` uses Kotlin native nullability in Kotlin modules and JSpecify
`1.0.1` at the Java extension boundary.

## Kotlin modules

Pure Kotlin modules use Kotlin's own nullability (`T` / `T?`). Kotlin metadata is the authoritative
source for Kotlin consumers.

A `package-info.java` is **not** required in a pure Kotlin module. JSpecify treats Kotlin classes
carrying `@kotlin.Metadata` as null-unmarked, so a package annotation alone would not make Kotlin
classes fully JSpecify-annotated.

If a Kotlin module later adds Java source or a Java-facing API in a package, add
`package-info.java` for that package at that point.

## Java extension

Every `pulselink-java` package contains `package-info.java`:

```java
@NullMarked
package top.chiloven.pulselink.java;

import org.jspecify.annotations.NullMarked;
```

Rules:

- Java API defaults to non-null; use `@Nullable` for nullable values.
- Prefer `Optional<T>` for optional value results.
- Never return nullable collections or nullable `CompletableFuture`.
- JSpecify is an `api` dependency so consumers see the annotations.

## Java analysis

`pulselink-java` runs NullAway in JSpecify mode:

```text
NullAway:OnlyNullMarked=true
NullAway:JSpecifyMode=true
```

JDK 21 requires `-XDaddTypeAnnotationsToSymbol=true`; use an OpenJDK build (Temurin is fine). The
checks must also run on JDK 25.
