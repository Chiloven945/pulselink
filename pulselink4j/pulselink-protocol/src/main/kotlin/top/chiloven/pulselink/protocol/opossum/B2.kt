package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8

private val B2_FIXED: IntArray = intArrayOf(
    0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x08, 0x09,
)

/** Display intensity sync command (`0xB2`). */
public data class B2(
    public val a: Int,
    public val b: Int,
) {

    public fun encode(): ByteArray =
        when {
            a !in 0..200 || b !in 0..200 -> {
                throw ProtocolException.ValueOutOfRange(
                    "Opossum display intensity",
                    maxOf(a, b).toLong(),
                    0,
                    200
                )
            }

            else -> ByteArray(24).also {
                it[0] = 0xB2.toU8()
                B2_FIXED.forEachIndexed { index, value -> it[1 + index] = value.toU8() }
                it[22] = a.toU8()
                it[23] = b.toU8()
            }
        }

}
