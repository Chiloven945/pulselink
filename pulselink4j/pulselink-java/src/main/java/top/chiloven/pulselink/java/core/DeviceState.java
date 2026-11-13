package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** An immutable snapshot of a device's observed state. */
public record DeviceState(
        long revision,
        ConnectionState connection,
        @Nullable BatteryState battery,
        ChannelState channelA,
        ChannelState channelB
) {

    public DeviceState {
        if (revision < 0) {
            throw new IllegalArgumentException("revision must not be negative");
        }
    }

    public ChannelState channel(Channel channel) {
        return switch (channel) {
            case A -> channelA;
            case B -> channelB;
        };
    }

}
