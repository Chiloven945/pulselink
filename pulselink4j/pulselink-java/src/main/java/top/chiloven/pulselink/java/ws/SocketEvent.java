package top.chiloven.pulselink.java.ws;

import org.jspecify.annotations.Nullable;

/** Socket lifecycle/control event. */
public sealed interface SocketEvent {

    record StateChanged(SocketState state) implements SocketEvent {

    }

    record ClientAttached(String clientId) implements SocketEvent {

    }

    record ClientDisconnected(String clientId) implements SocketEvent {

    }

    record Heartbeat() implements SocketEvent {

    }

    record Pong(@Nullable Long ts) implements SocketEvent {

    }

    record IdleTimeout() implements SocketEvent {

    }

    record Closed(
            @Nullable Integer code,
            String reason
    ) implements SocketEvent {

    }

    record Failure(String message) implements SocketEvent {

    }

}
