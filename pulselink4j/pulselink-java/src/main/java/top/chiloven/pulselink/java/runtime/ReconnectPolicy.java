package top.chiloven.pulselink.java.runtime;

import java.time.Duration;

/** Reconnect policy. */
public record ReconnectPolicy(
        boolean enabled,
        int maxAttempts,
        Duration initialBackoff,
        Duration maxBackoff,
        boolean jitter
) {

    public ReconnectPolicy {
        if (maxAttempts < 0) {
            throw new IllegalArgumentException("maxAttempts must not be negative");
        }
        if (initialBackoff.isNegative() || maxBackoff.isNegative()) {
            throw new IllegalArgumentException("backoff must not be negative");
        }
    }

    public static ReconnectPolicy defaults() {
        return new ReconnectPolicy(
                true,
                5,
                Duration.ofMillis(500),
                Duration.ofSeconds(30),
                true
        );
    }

}
