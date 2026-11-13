package top.chiloven.pulselink.protocol.coyote.v3

/** Intensity operations for both channels. */
public data class StereoIntensityOp(
    public val a: IntensityOp,
    public val b: IntensityOp,
)
