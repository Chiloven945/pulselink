package top.chiloven.pulselink.pattern

/** One 25 ms sample of the waveform timeline. */
public data class PulseSample(
    public val interval: PulseInterval,
    public val intensity: PulseIntensity,
)
