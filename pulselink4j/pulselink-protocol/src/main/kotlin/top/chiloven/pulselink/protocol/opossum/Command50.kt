package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.toU8

/** Indicator color and button-reporting command (`0x50`). */
public data class Command50(
    public val color: Int,
    public val buttonReport: Boolean,
) {

    public fun encode(): ByteArray =
        byteArrayOf(
            0x50.toU8(),
            color.toU8(),
            (if (buttonReport) 1 else 0).toU8()
        )

}
