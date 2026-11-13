package top.chiloven.pulselink.java.ws;

import java.time.Duration;
import org.jspecify.annotations.Nullable;

/** WebSocket connection options. */
public record SocketOptions(
        String url,
        Duration connectTimeout,
        Duration responseTimeout,
        @Nullable Duration pingInterval,
        long maxFrameSize
) {

    public SocketOptions {
        if (url.isBlank()) {
            throw new IllegalArgumentException("url must not be blank");
        }
        if (connectTimeout.isNegative() || responseTimeout.isNegative()) {
            throw new IllegalArgumentException("timeouts must not be negative");
        }
        if (maxFrameSize <= 0) {
            throw new IllegalArgumentException("maxFrameSize must be positive");
        }
    }

    public static SocketOptions defaults(String url) {
        return new SocketOptions(
                url,
                Duration.ofSeconds(10),
                Duration.ofSeconds(8),
                null,
                8L * 1024 * 1024
        );
    }

}
