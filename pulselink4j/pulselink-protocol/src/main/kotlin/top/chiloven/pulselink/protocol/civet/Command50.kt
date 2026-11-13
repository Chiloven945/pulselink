package top.chiloven.pulselink.protocol.civet

import top.chiloven.pulselink.protocol.toU8

/** Start or stop Civet pressure reporting. */
public data class Command50(
    public val color: Int,
    public val start: Boolean,
) {

    /** Encodes the 17-byte command. */
    public fun encode(): ByteArray =
        ByteArray(17).also {
            it[0] = 0x50.toU8()
            it[1] = color.toU8()
            it[2] = (if (start) 0xD0 else 0x00).toU8()
        }

}
