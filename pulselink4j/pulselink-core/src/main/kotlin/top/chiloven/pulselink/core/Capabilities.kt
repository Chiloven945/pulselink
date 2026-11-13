package top.chiloven.pulselink.core

import kotlin.time.Duration

/** The set of channels a device exposes. */
public data class ChannelSet(
    public val a: Boolean,
    public val b: Boolean,
) {

    /** Returns whether the set contains [channel]. */
    public fun contains(channel: Channel): Boolean =
        when (channel) {
            Channel.A -> a
            Channel.B -> b
        }

    /** Returns the number of channels in the set. */
    public fun count(): Int =
        (if (a) 1 else 0) + (if (b) 1 else 0)

    public companion object {

        public val NONE: ChannelSet = ChannelSet(a = false, b = false)
        public val A: ChannelSet = ChannelSet(a = true, b = false)
        public val B: ChannelSet = ChannelSet(a = false, b = true)
        public val AB: ChannelSet = ChannelSet(a = true, b = true)

    }

}

/** Intensity control capabilities. */
public data class IntensityCapabilities(
    public val absolute: Boolean = false,
    public val relative: Boolean = false,
    public val temporary: Boolean = false,
    public val max: Intensity = Intensity.MIN,
)

/** Waveform capabilities. */
public data class WaveformCapabilities(
    public val v2Frames: Boolean = false,
    public val v3Frames: Boolean = false,
    public val maxDuration: Duration? = null,
)

/** A summary of what a device can do. */
public data class DeviceCapabilities(
    public val channels: ChannelSet = ChannelSet.NONE,
    public val battery: Boolean = false,
    public val intensity: IntensityCapabilities = IntensityCapabilities(),
    public val waveform: WaveformCapabilities = WaveformCapabilities(),
    public val settings: Boolean = false,
    public val pressure: Boolean = false,
    public val buttons: Boolean = false,
    public val vibration: Boolean = false,
)
