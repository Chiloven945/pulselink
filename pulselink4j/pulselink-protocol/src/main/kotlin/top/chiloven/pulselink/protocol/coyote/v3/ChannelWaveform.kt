package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.pattern.PulseSample
import top.chiloven.pulselink.protocol.ProtocolException

/** Waveform data for one channel in a 100 ms B0 frame. */
public sealed interface ChannelWaveform {

    public data class Active(
        public val samples: List<PulseSample>
    ) : ChannelWaveform {

        init {
            if (samples.size != 4) {
                throw ProtocolException.InvalidPulseFrame("active waveform requires exactly 4 samples")
            }
        }
    }

    public data object Silent : ChannelWaveform

    public companion object {

        public fun active(
            first: PulseSample,
            second: PulseSample,
            third: PulseSample,
            fourth: PulseSample
        ): ChannelWaveform =
            Active(listOf(first, second, third, fourth))

    }

}
