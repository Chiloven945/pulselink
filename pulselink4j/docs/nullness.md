# JSpecify nullness

`pulselink4j` uses JSpecify `1.0.1` and `@NullMarked` at package level.

## Package rule

Every Java-visible package contains `package-info.java`:

```java
@NullMarked
package top.chiloven.pulselink.<module>.<package>;

import org.jspecify.annotations.NullMarked;
```

This applies to every package in `pulselink-java`, and to Kotlin modules that expose Java-facing
packages.

## Kotlin boundary

Kotlin code keeps Kotlin native nullability (`T` / `T?`) as the primary contract. JSpecify is the
Java boundary:

- Java extension returns `@Nullable T` only when null is meaningful.
- Prefer `Optional<T>` for optional value results.
- Never return nullable collections or nullable `CompletableFuture`.
- JSpecify is an `api` dependency so consumers see the annotations.

## NullAway

`pulselink-java` runs NullAway in JSpecify mode:

```text
NullAway:OnlyNullMarked=true
NullAway:JSpecifyMode=true
```

JDK 21 requires `-XDaddTypeAnnotationsToSymbol=true` and an OpenJDK build. CI also runs the Java
extension checks on JDK 25.
