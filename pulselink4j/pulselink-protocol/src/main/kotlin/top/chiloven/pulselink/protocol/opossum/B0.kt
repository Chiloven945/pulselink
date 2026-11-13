package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8

/** Dual-channel vibration frame (`0xB0`). */
public data class B0(
    public val a: List<Int>,
    public val b: List<Int>,
) {

    init {
        when {
            a.size != 4 || b.size != 4 ->
                throw ProtocolException.InvalidParameters("Opossum B0 requires 4 values per channel")

            (a + b).any { it !in 0..100 } ->
                throw ProtocolException.ValueOutOfRange(
                    "Opossum vibration intensity",
                    101,
                    0,
                    100
                )
        }
    }

    public fun encode(): ByteArray =
        ByteArray(20).also {
            it[0] = 0xB0.toU8()
            a.forEachIndexed { index, value -> it[8 + index] = value.toU8() }
            b.forEachIndexed { index, value -> it[16 + index] = value.toU8() }
        }

}
