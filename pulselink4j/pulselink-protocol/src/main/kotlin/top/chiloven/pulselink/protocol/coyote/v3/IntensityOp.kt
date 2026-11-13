package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.core.Intensity

/** The intensity operation for one channel. */
public sealed interface IntensityOp {

    public data object Keep : IntensityOp

    public data class Increase(public val value: Intensity) : IntensityOp

    public data class Decrease(public val value: Intensity) : IntensityOp

    public data class Set(public val value: Intensity) : IntensityOp

}
