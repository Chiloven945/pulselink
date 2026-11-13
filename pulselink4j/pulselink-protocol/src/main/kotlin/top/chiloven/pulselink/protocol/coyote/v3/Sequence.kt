package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.protocol.ProtocolException

/** A 4-bit B0 sequence number. */
@JvmInline
public value class Sequence private constructor(public val value: Int) {

    public companion object {

        public val ZERO: Sequence = Sequence(0)
        public const val MAX: Int = 15

        public fun of(value: Int): Sequence =
            when (value) {
                in 0..MAX -> Sequence(value)
                else -> throw ProtocolException.InvalidSequence(value)
            }

    }

}
