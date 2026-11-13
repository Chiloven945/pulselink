package top.chiloven.pulselink.core

/**
 * A bitmask of physical buttons.
 */
@JvmInline
public value class ButtonMask private constructor(public val bits: Int) {

    /** Returns whether every bit in [other] is set. */
    public fun contains(other: ButtonMask): Boolean =
        bits and other.bits == other.bits

    /** Returns whether any bit is set. */
    public fun isEmpty(): Boolean =
        bits == 0

    public companion object {

        public val NONE: ButtonMask = ButtonMask(0)

        /** Creates a mask from raw bits. */
        public fun fromBits(bits: Int): ButtonMask =
            when (bits) {
                in 0..0xFFFF -> ButtonMask(bits)
                else -> throw CoreException.InvalidButtonMask(bits)
            }

    }

}

/**
 * A parameterized trigger event reported by a sensor.
 */
public data class Trigger(
    public val eventId: Int,
    public val parameter: Int,
) {

    init {
        if (eventId !in 0..255 || parameter !in 0..255) {
            throw CoreException.InvalidTrigger(eventId, parameter)
        }
    }

}
