package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.pattern.PulseIntensity
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.pattern.PulseSample
import top.chiloven.pulselink.protocol.coyote.v3.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlinx.serialization.json.*

class V3VectorTest {

    private val vectors = VectorLoader()

    private fun intensityOp(value: JsonObject): IntensityOp =
        when (val op = value.getValue("op").jsonPrimitive.content) {
            "keep" -> IntensityOp.Keep
            "increase" -> IntensityOp.Increase(Intensity.of(value.getValue("value").jsonPrimitive.int))
            "decrease" -> IntensityOp.Decrease(Intensity.of(value.getValue("value").jsonPrimitive.int))
            "set" -> IntensityOp.Set(Intensity.of(value.getValue("value").jsonPrimitive.int))
            else -> error("unknown intensity op $op")
        }

    private fun waveform(value: JsonObject): ChannelWaveform =
        when (value.getValue("kind").jsonPrimitive.content) {
            "silent" -> ChannelWaveform.Silent
            "active" -> {
                val frequencies = value.getValue("frequencies").jsonArray
                val intensities = value.getValue("intensities").jsonArray
                ChannelWaveform.Active(
                    frequencies.mapIndexed { index, frequency ->
                        PulseSample(
                            PulseInterval.of(frequency.jsonPrimitive.int),
                            PulseIntensity.of(intensities[index].jsonPrimitive.int),
                        )
                    },
                )
            }

            else -> error("unknown waveform")
        }

    @Test
    fun v3B0Vectors() {
        for (case in vectors.cases("coyote-v3/b0.json")) {
            val input = case.getValue("input").jsonObject
            val command = B0Command(
                sequence = Sequence.of(input.getValue("sequence").jsonPrimitive.int),
                intensity = StereoIntensityOp(
                    intensityOp(input.getValue("intensity_a").jsonObject),
                    intensityOp(input.getValue("intensity_b").jsonObject)
                ),
                waveform = StereoWaveform(
                    waveform(input.getValue("waveform_a").jsonObject),
                    waveform(input.getValue("waveform_b").jsonObject)
                ),
            )
            assertEquals(
                case.getValue("expect").jsonObject.getValue("hex").jsonPrimitive.content,
                vectors.hex(command.encode()),
                case.getValue("id").jsonPrimitive.content
            )
        }
    }

    @Test
    fun v3BfVectors() {
        for (case in vectors.cases("coyote-v3/bf.json")) {
            val input = case.getValue("input").jsonObject
            val command = BfCommand(
                softLimit = ChannelMap(
                    Intensity.of(input.getValue("soft_limit_a").jsonPrimitive.int),
                    Intensity.of(input.getValue("soft_limit_b").jsonPrimitive.int)
                ),
                frequencyBalance = ChannelMap(
                    input.getValue("frequency_balance_a").jsonPrimitive.int,
                    input.getValue("frequency_balance_b").jsonPrimitive.int
                ),
                intensityBalance = ChannelMap(
                    input.getValue("intensity_balance_a").jsonPrimitive.int,
                    input.getValue("intensity_balance_b").jsonPrimitive.int
                ),
            )
            assertEquals(
                case.getValue("expect").jsonObject.getValue("hex").jsonPrimitive.content,
                vectors.hex(command.encode()),
                case.getValue("id").jsonPrimitive.content
            )
        }
    }

    @Test
    fun v3B1Vectors() {
        for (case in vectors.cases("coyote-v3/b1.json")) {
            val input = case.getValue("input").jsonObject.getValue("hex").jsonPrimitive.content
            val expect = case.getValue("expect").jsonObject
            if (expect["error"]?.jsonPrimitive?.content?.toBoolean() == true) {
                assertFails { B1Message.decode(vectors.unhex(input)) }
            } else {
                val message = B1Message.decode(vectors.unhex(input))
                assertEquals(
                    expect.getValue("sequence").jsonPrimitive.int,
                    message.sequence.value
                )
                assertEquals(
                    expect.getValue("a").jsonPrimitive.int,
                    message.intensity.a.value
                )
                assertEquals(
                    expect.getValue("b").jsonPrimitive.int,
                    message.intensity.b.value
                )
            }
        }
    }

    @Test
    fun v3FrequencyVectors() {
        for (case in vectors.cases("coyote-v3/frequency.json")) {
            val input = case.getValue("input").jsonObject
            val expect = case.getValue("expect").jsonObject
            when (case.getValue("kind").jsonPrimitive.content) {
                "frequency.encode" -> assertEquals(
                    expect.getValue("wire").jsonPrimitive.int,
                    encodeInterval(
                        PulseInterval.of(input.getValue("interval_ms").jsonPrimitive.int)
                    ),
                )

                "frequency.decode" -> assertEquals(
                    expect.getValue("interval_ms").jsonPrimitive.int,
                    decodeWireFrequency(
                        input.getValue("wire").jsonPrimitive.int
                    ).milliseconds,
                )
            }
        }
    }

}
