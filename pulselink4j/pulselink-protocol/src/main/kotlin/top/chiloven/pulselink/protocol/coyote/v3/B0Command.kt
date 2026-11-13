package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.protocol.toU8

/** A Coyote V3 B0 control command. */
public data class B0Command(
    public val sequence: Sequence,
    public val intensity: StereoIntensityOp,
    public val waveform: StereoWaveform,
) {

    /** Encodes the command into 20 bytes. */
    public fun encode(): ByteArray {
        val output = ByteArray(20)
        val (modeA, intensityA) = encodeIntensityOp(intensity.a)
        val (modeB, intensityB) = encodeIntensityOp(intensity.b)

        output[0] = 0xB0.toU8()
        output[1] = ((sequence.value shl 4) or (modeA shl 2) or modeB).toU8()
        output[2] = intensityA.toU8()
        output[3] = intensityB.toU8()

        encodeWaveform(waveform.a).copyInto(output, 4)
        encodeWaveform(waveform.b).copyInto(output, 12)

        return output
    }

}

private fun encodeIntensityOp(op: IntensityOp): Pair<Int, Int> =
    when (op) {
        IntensityOp.Keep -> 0b00 to 0
        is IntensityOp.Increase -> 0b01 to op.value.value
        is IntensityOp.Decrease -> 0b10 to op.value.value
        is IntensityOp.Set -> 0b11 to op.value.value
    }

private fun encodeWaveform(waveform: ChannelWaveform): ByteArray {
    val output = ByteArray(8)
    when (waveform) {
        is ChannelWaveform.Active -> {
            waveform.samples.forEachIndexed { index, sample ->
                output[index] = encodeInterval(sample.interval).toU8()
                output[4 + index] = sample.intensity.value.toU8()
            }
        }

        ChannelWaveform.Silent -> {
            output.fill(0)
            output[7] = 101.toU8()
        }
    }
    return output
}
