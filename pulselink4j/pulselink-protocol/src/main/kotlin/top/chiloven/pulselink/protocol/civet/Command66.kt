package top.chiloven.pulselink.protocol.civet

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8

/** Reset pressure and/or set display orientation. */
public data class Command66(
    public val orientation: Int? = null,
    public val reset: Boolean = false,
) {

    /** Encodes the 13-byte command. */
    public fun encode(): ByteArray =
        when {
            orientation != null && orientation !in listOf(1, 3) ->
                throw ProtocolException.ValueOutOfRange(
                    "Civet orientation",
                    orientation.toLong(),
                    1,
                    3
                )

            else ->
                ByteArray(13).also {
                    it[0] = 0x66.toU8()
                    it[10] = (orientation ?: 0).toU8()
                    it[12] = (if (reset) 2 else 0).toU8()
                }
        }

}
