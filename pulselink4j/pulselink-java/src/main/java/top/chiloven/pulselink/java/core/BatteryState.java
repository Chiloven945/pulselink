package top.chiloven.pulselink.java.core;

/** Observed battery information. */
public record BatteryState(int level) {

    public BatteryState {
        if (level < 0 || level > 100) {
            throw new IllegalArgumentException("battery level must be 0..100");
        }
    }

}
