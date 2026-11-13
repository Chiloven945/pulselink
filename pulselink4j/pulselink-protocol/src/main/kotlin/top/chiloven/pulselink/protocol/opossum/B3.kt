package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8

/** Channel intensity command (`0xB3`); null leaves a channel unchanged. */
public data class B3(
    public val a: Int?,
    public val b: Int?,
) {

    public fun encode(): ByteArray =
        byteArrayOf(0xB3.toU8(), encoded(a), encoded(b))

    private fun encoded(value: Int?): Byte =
        when (value) {
            null -> 0xFF.toU8()

            !in 0..200 -> throw ProtocolException.ValueOutOfRange(
                "Opossum intensity",
                value.toLong(),
                0,
                200
            )

            else -> value.toU8()
        }

}
