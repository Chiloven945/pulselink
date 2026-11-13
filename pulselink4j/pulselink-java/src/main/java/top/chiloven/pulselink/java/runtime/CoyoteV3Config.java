package top.chiloven.pulselink.java.runtime;

import java.time.Duration;
import org.jspecify.annotations.Nullable;

/** Coyote V3 session configuration. */
public final class CoyoteV3Config {

    private final Duration ackTimeout;
    private final Duration tickInterval;
    private final Duration lagThreshold;
    private final SafetyPolicy safety;
    private final ReconnectPolicy reconnect;
    private final int initialSoftLimitA;
    private final int initialSoftLimitB;
    private final @Nullable CoyoteV3Balance balance;

    private CoyoteV3Config(Builder builder) {
        this.ackTimeout = builder.ackTimeout;
        this.tickInterval = builder.tickInterval;
        this.lagThreshold = builder.lagThreshold;
        this.safety = builder.safety;
        this.reconnect = builder.reconnect;
        this.initialSoftLimitA = builder.initialSoftLimitA;
        this.initialSoftLimitB = builder.initialSoftLimitB;
        this.balance = builder.balance;
        requireIntensity("initialSoftLimitA", initialSoftLimitA);
        requireIntensity("initialSoftLimitB", initialSoftLimitB);
    }

    private static void requireIntensity(String name, int value) {
        if (value < 0 || value > 200) {
            throw new IllegalArgumentException(name + " must be 0..200");
        }
    }

    public static CoyoteV3Config defaults() {
        return builder().build();
    }

    public static Builder builder() {
        return new Builder();
    }

    public Duration ackTimeout() {
        return ackTimeout;
    }

    public Duration tickInterval() {
        return tickInterval;
    }

    public Duration lagThreshold() {
        return lagThreshold;
    }

    public SafetyPolicy safety() {
        return safety;
    }

    public ReconnectPolicy reconnect() {
        return reconnect;
    }

    public int initialSoftLimitA() {
        return initialSoftLimitA;
    }

    public int initialSoftLimitB() {
        return initialSoftLimitB;
    }

    public @Nullable CoyoteV3Balance balance() {
        return balance;
    }

    /** Builder for {@link CoyoteV3Config}. */
    public static final class Builder {

        private Duration ackTimeout = Duration.ofSeconds(1);
        private Duration tickInterval = Duration.ofMillis(100);
        private Duration lagThreshold = Duration.ofMillis(500);
        private SafetyPolicy safety = SafetyPolicy.defaults();
        private ReconnectPolicy reconnect = ReconnectPolicy.defaults();
        private int initialSoftLimitA = 200;
        private int initialSoftLimitB = 200;
        private @Nullable CoyoteV3Balance balance;

        private Builder() {
        }

        public Builder ackTimeout(Duration value) {
            this.ackTimeout = value;
            return this;
        }

        public Builder tickInterval(Duration value) {
            this.tickInterval = value;
            return this;
        }

        public Builder lagThreshold(Duration value) {
            this.lagThreshold = value;
            return this;
        }

        public Builder safety(SafetyPolicy value) {
            this.safety = value;
            return this;
        }

        public Builder reconnect(ReconnectPolicy value) {
            this.reconnect = value;
            return this;
        }

        public Builder initialSoftLimit(int channelA, int channelB) {
            this.initialSoftLimitA = channelA;
            this.initialSoftLimitB = channelB;
            return this;
        }

        public Builder balance(@Nullable CoyoteV3Balance value) {
            this.balance = value;
            return this;
        }

        public CoyoteV3Config build() {
            return new CoyoteV3Config(this);
        }

    }

}
