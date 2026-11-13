package top.chiloven.pulselink.protocol.coyote.v3

/** Waveform data for both channels. */
public data class StereoWaveform(
    public val a: ChannelWaveform,
    public val b: ChannelWaveform,
) {

    public companion object {

        public fun silent(): StereoWaveform =
            StereoWaveform(ChannelWaveform.Silent, ChannelWaveform.Silent)

    }

}
