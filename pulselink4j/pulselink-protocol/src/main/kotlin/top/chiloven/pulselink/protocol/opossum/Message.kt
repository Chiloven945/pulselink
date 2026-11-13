package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.u8

/** A decoded Opossum notification. */
public sealed interface Message {

    public data class B3Value(public val message: B3Message) : Message
    public data class D0Value(public val message: D0Message) : Message

    public companion object {

        public fun decode(bytes: ByteArray): Message =
            when (bytes.firstOrNull()?.toInt()?.and(0xFF)) {
                0xB3 -> B3Value(B3Message.decode(bytes))
                0xD0 -> D0Value(D0Message.decode(bytes))
                null -> throw ProtocolException.InvalidLength(3, 0)
                else -> throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

    }

}
