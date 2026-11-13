package top.chiloven.pulselink.pattern

import top.chiloven.pulselink.core.Channel

/** A 100 ms frame containing both channels. */
public data class StereoFrame(
    public val a: ChannelFrame,
    public val b: ChannelFrame,
) {

    /** Returns the frame for [channel]. */
    public fun channel(channel: Channel): ChannelFrame =
        when (channel) {
            Channel.A -> a
            Channel.B -> b
        }

    public companion object {

        public fun silent(): StereoFrame =
            StereoFrame(ChannelFrame.Silent, ChannelFrame.Silent)

    }

}
