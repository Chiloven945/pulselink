package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.PulseLinkException

/** Errors returned by protocol encoding, decoding, and validation. */
public sealed class ProtocolException(message: String) : PulseLinkException(message) {

    public data class InvalidLength(
        public val expected: Int,
        public val actual: Int
    ) : ProtocolException("invalid frame length: expected $expected, got $actual")

    public data class InvalidHeader(
        public val value: Int
    ) : ProtocolException("invalid header: 0x${value.toString(16).padStart(2, '0')}")

    public data class InvalidSequence(
        public val value: Int
    ) : ProtocolException("invalid sequence $value; expected 0..15")

    public data class ValueOutOfRange(
        public val field: String,
        public val value: Long,
        public val min: Long,
        public val max: Long,
    ) : ProtocolException("value out of range for $field: $value (expected $min..=$max)")

    public data class InvalidPulseFrame(
        public val reason: String
    ) : ProtocolException("invalid pulse frame: $reason")

    public data class UnsupportedActionType(
        public val value: Int
    ) : ProtocolException("unsupported V4 action type: $value")

    public data class InvalidParameters(
        public val reason: String
    ) : ProtocolException("invalid parameters: $reason")

    public data class Json(
        public val reason: String
    ) : ProtocolException("invalid JSON: $reason")

}
