package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.core.ButtonMask
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** An Opossum `0xD0` sensor notification. */
public data class D0Message(
    public val sequence: Int,
    public val buttons: ButtonMask,
    public val raw: ByteArray,
) {

    public companion object {

        public fun decode(bytes: ByteArray): D0Message {
            bytes.requireLength(16)
            if (bytes.u8(0) != 0xD0) {
                throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

            val bits = bytes.u8(2) or (bytes.u8(3) shl 8)
            return D0Message(
                bytes.u8(1),
                ButtonMask.fromBits(bits),
                bytes.copyOf()
            )
        }

    }

}
