package top.chiloven.pulselink.protocol.pawprints

import top.chiloven.pulselink.protocol.toU8

public val COMMAND_RESET_PARAMETER: ByteArray = byteArrayOf(0x5F.toU8())
public val COMMAND_AUTO_DETECT_ANGLES: ByteArray = byteArrayOf(0x60.toU8())

public fun commandShoulderLight(color: Int): ByteArray =
    byteArrayOf(
        0x70.toU8(),
        color.toU8()
    )

public fun commandShoulderLightFlash(
    colorA: Int,
    colorB: Int,
    speed: Int
): ByteArray =
    byteArrayOf(
        0x70.toU8(),
        colorA.toU8(),
        colorB.toU8(),
        speed.toU8()
    )
