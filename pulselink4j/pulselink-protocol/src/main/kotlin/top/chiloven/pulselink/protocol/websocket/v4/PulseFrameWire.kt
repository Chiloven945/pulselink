package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException

/** A pulse frame in hexadecimal or numeric form. */
public sealed interface PulseFrameWire {

    public data class Hex(public val value: String) : PulseFrameWire

    public data class Numbers(public val values: List<Int>) : PulseFrameWire

    /** Validates the byte length and hex encoding. */
    public fun validate(expectedBytes: Int) {
        when (this) {
            is Hex -> when {
                value.length != expectedBytes * 2 ->
                    throw ProtocolException.InvalidPulseFrame("hex frame has the wrong byte length")

                !value.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' } ->
                    throw ProtocolException.InvalidPulseFrame("hex frame contains a non-hex character")
            }

            is Numbers -> when {
                values.size != expectedBytes ->
                    throw ProtocolException.InvalidPulseFrame("numeric frame has the wrong length")

                values.any { it !in 0..255 } ->
                    throw ProtocolException.InvalidPulseFrame("numeric frame contains a non-byte value")
            }

        }
    }

}
