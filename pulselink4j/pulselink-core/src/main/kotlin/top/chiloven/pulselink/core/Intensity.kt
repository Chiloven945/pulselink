package top.chiloven.pulselink.core

/**
 * An absolute intensity in the range `0..=200`.
 */
@JvmInline
public value class Intensity private constructor(public val value: Int) : Comparable<Intensity> {

    override fun compareTo(other: Intensity): Int =
        value.compareTo(other.value)

    override fun toString(): String =
        value.toString()

    public companion object {

        public val MIN: Intensity = Intensity(0)
        public val MAX: Intensity = Intensity(200)

        /** Creates a validated intensity. */
        public fun of(value: Int): Intensity =
            when (value) {
                in 0..200 -> Intensity(value)
                else -> throw CoreException.InvalidIntensity(value)
            }

    }

    /** Adds a delta, returning null when the result would leave the valid range. */
    public fun checkedAdd(delta: Int): Intensity? =
        (value + delta).takeIf {
            it in 0..200
        }?.let(::Intensity)

    /** Subtracts a delta, returning null when the result would leave the valid range. */
    public fun checkedSub(delta: Int): Intensity? =
        checkedAdd(-delta)

    /** Adds a delta, clamping to the valid range. */
    public fun saturatingAdd(delta: Int): Intensity =
        Intensity((value + delta).coerceIn(0, 200))

    /** Subtracts a delta, clamping to the valid range. */
    public fun saturatingSub(delta: Int): Intensity =
        saturatingAdd(-delta)

}

/**
 * A relative intensity delta in the range `-200..=200`.
 */
@JvmInline
public value class IntensityDelta private constructor(
    public val value: Int
) : Comparable<IntensityDelta> {

    override fun compareTo(other: IntensityDelta): Int =
        value.compareTo(other.value)

    override fun toString(): String =
        value.toString()

    public companion object {

        public val MIN: IntensityDelta = IntensityDelta(-200)
        public val MAX: IntensityDelta = IntensityDelta(200)

        /** Creates a validated delta. */
        public fun of(value: Int): IntensityDelta =
            when (value) {
                in -200..200 -> IntensityDelta(value)
                else -> throw CoreException.InvalidIntensityDelta(value)
            }

    }

    /** Returns the absolute magnitude. */
    public fun magnitude(): Int =
        kotlin.math.abs(value)

}
