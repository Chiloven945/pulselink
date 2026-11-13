package top.chiloven.pulselink.java.pattern;

import top.chiloven.pulselink.java.core.Channel;

/** A 100 ms frame containing both channels. */
public record StereoFrameSpec(
        ChannelFrameSpec a,
        ChannelFrameSpec b
) {

    public static StereoFrameSpec silent() {
        return new StereoFrameSpec(
                ChannelFrameSpec.silent(),
                ChannelFrameSpec.silent()
        );
    }

    public ChannelFrameSpec channel(Channel channel) {
        return switch (channel) {
            case A -> a;
            case B -> b;
        };
    }

}
