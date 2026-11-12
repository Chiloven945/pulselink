package top.chiloven.pulselink.pattern

import top.chiloven.pulselink.core.Channel

/** The waveform data for one channel in a 100 ms frame. */
public sealed interface ChannelFrame {

    /** Four 25 ms samples of active waveform data. */
    public data class Active(public val samples: List<PulseSample>) : ChannelFrame {

        init {
            if (samples.size != SAMPLES_PER_FRAME) {
                throw PatternException.InvalidSampleCount(samples.size)
            }
        }

        public companion object {

            public const val SAMPLES_PER_FRAME: Int = 4

        }

    }

    /** Explicit silence for the whole 100 ms window. */
    public data object Silent : ChannelFrame

    public companion object {

        public fun active(
            first: PulseSample,
            second: PulseSample,
            third: PulseSample,
            fourth: PulseSample
        ): ChannelFrame =
            Active(listOf(first, second, third, fourth))

    }

}

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
