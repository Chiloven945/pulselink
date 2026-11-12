package top.chiloven.pulselink.protocol

internal fun ByteArray.u8(index: Int): Int =
    this[index].toInt() and 0xFF

internal fun Int.toU8(): Byte =
    (this and 0xFF).toByte()

internal fun ByteArray.requireLength(expected: Int) {
    if (size != expected) {
        throw ProtocolException.InvalidLength(expected, size)
    }
}
