package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8

/** A Coyote V3 BF settings command. */
public data class BfCommand(
    public val softLimit: ChannelMap<Intensity>,
    public val frequencyBalance: ChannelMap<Int>,
    public val intensityBalance: ChannelMap<Int>,
) {

    init {
        validateBalance("BF frequency balance A", frequencyBalance.a)
        validateBalance("BF frequency balance B", frequencyBalance.b)
        validateBalance("BF intensity balance A", intensityBalance.a)
        validateBalance("BF intensity balance B", intensityBalance.b)
    }

    /** Encodes the command into 7 bytes. */
    public fun encode(): ByteArray = byteArrayOf(
        0xBF.toU8(),
        softLimit.a.value.toU8(),
        softLimit.b.value.toU8(),
        frequencyBalance.a.toU8(),
        frequencyBalance.b.toU8(),
        intensityBalance.a.toU8(),
        intensityBalance.b.toU8(),
    )

}

private fun validateBalance(field: String, value: Int) {
    if (value !in 0..255) {
        throw ProtocolException.ValueOutOfRange(field, value.toLong(), 0, 255)
    }
}
