# Java extension

`pulselink-java` is the optional Java 21 facade over the Kotlin modules. It uses Java interfaces,
records, sealed event types, `CompletableFuture`, `java.util.concurrent.Flow.Publisher`, and
`AutoCloseable`.

## Dependency

```text
top.chiloven:pulselink-java:0.1.0
```

The Java artifact depends transitively on the Kotlin runtime, device, BLE, pattern, and WebSocket
modules. Java 21 or newer is required.

## Local devices

```java
import top.chiloven.pulselink.java.PulseLinkClient;
import top.chiloven.pulselink.java.core.Channel;
import top.chiloven.pulselink.java.core.DeviceState;
import top.chiloven.pulselink.java.device.CoyoteV3Session;
import top.chiloven.pulselink.java.transport.DiscoveredDevice;
import top.chiloven.pulselink.java.transport.ScanFilter;

import java.time.Duration;
import java.util.List;
import java.util.concurrent.Flow;

public final class LocalDeviceExample {

    public static void main(String[] args) {
        try (var client = new PulseLinkClient()) {
            List<DiscoveredDevice> devices = client
                    .scan(ScanFilter.namePrefix("47L121000"), Duration.ofSeconds(5))
                    .join();

            if (devices.isEmpty()) {
                System.out.println("no matching device found");
                return;
            }

            try (
                    CoyoteV3Session session = client
                            .connectCoyoteV3(devices.getFirst())
                            .join()
            ) {
                session.states().subscribe(new StateSubscriber());
                session.setIntensity(Channel.A, 10).join();
                session.shutdown().join();
            }
        }
    }

    private static final class StateSubscriber implements Flow.Subscriber<DeviceState> {

        @Override
        public void onSubscribe(Flow.Subscription subscription) {
            subscription.request(Long.MAX_VALUE);
        }

        @Override
        public void onNext(DeviceState state) {
            System.out.println("state revision: " + state.revision());
        }

        @Override
        public void onError(Throwable error) {
            error.printStackTrace();
        }

        @Override
        public void onComplete() {
        }

    }

}
```

`PulseLinkClient.connect` auto-detects Coyote V2/V3, Opossum, Civet, and PawPrints. Use
`connectCoyoteV3` when the full runtime API is needed.

## DG-LAB V4 WebSocket

```java
import top.chiloven.pulselink.java.ws.AppClient;
import top.chiloven.pulselink.java.ws.RemoteDevice;
import top.chiloven.pulselink.java.ws.Socket;

import java.util.List;

public final class WebSocketExample {

    public static void main(String[] args) {
        try (var socket = Socket.connect("wss://relay.example/v4").join()) {
            System.out.println("target id: " + socket.targetId());

            var app = socket.waitForClient().join();
            var devices = app.requestDevices().join();
            System.out.println("device count: " + devices.size());
            app.ping().join();

            socket.shutdown().join();
        }
    }

}
```

## Async model

Every I/O method returns a non-null `CompletableFuture<T>`.

- `join()` / `get()` wrap failures in `CompletionException`; the cause is a
  `PulseLinkException` subtype.
- `PulseLinkException` is unchecked. Kotlin checked exceptions are mapped at the boundary.
- Cancelling the returned future cancels the underlying coroutine where possible.
- Completion callbacks run on the SDK dispatcher; do not block inside them.

## State and events

- `state()` returns the current immutable snapshot.
- `states()` returns a `Flow.Publisher<DeviceState>`; `StateFlow` semantics replay the current value
  to a new collector and conflate slow subscribers.
- `events()` returns a `Flow.Publisher<LocalEvent>`; events are best-effort and are not replayed.
- `CoyoteV3Session.runtimeStates()` / `runtimeEvents()` expose the richer runtime state and events.

Publishers use JDK `SubmissionPublisher` with a virtual-thread executor. Subscriber callbacks must
be thread-safe.

## Lifecycle

- `close()` is synchronous and best-effort.
- `shutdown()` is asynchronous and returns when sessions/sockets have been shut down.
- Both are idempotent.
- Prefer `shutdown().join()` before `close()` when graceful teardown matters.

## Nullness

All public packages are `@NullMarked`. Optional fields use `@Nullable`; optional lookup results use
`Optional<T>`. Public API types never expose Kotlin value classes, `Flow`, `StateFlow`, or
`kotlin.time.Duration`.

## Implementation structure

The public API lives in `src/main/java`. Kotlin adapters live in
`src/main/kotlin/top/chiloven/pulselink/java/internal` and implement the Java ports. The adapters
are not part of the public API or API dump; Javadoc only covers the Java sources.
