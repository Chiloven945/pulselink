package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.BatteryLevel
import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.pattern.PulseIntensity
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.pattern.PulseSample
import top.chiloven.pulselink.protocol.coyote.v3.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class V3Test {

    private fun sample(millis: Int, intensity: Int): PulseSample =
        PulseSample(
            PulseInterval.of(millis),
            PulseIntensity.of(intensity),
        )

    @Test
    fun officialB0Hex() {
        val command = B0Command(
            sequence = Sequence.of(1),
            intensity = StereoIntensityOp(
                a = IntensityOp.Set(Intensity.of(10)),
                b = IntensityOp.Keep,
            ),
            waveform = StereoWaveform(
                a = ChannelWaveform.Active(
                    listOf(
                        sample(100, 20),
                        sample(100, 30),
                        sample(100, 20),
                        sample(100, 10)
                    ),
                ),
                b = ChannelWaveform.Silent,
            ),
        )
        assertEquals(
            "B01C0A0064646464141E140A0000000000000065",
            command.encode()
                    .joinToString("") {
                        (it.toInt() and 0xFF).toString(16)
                                .padStart(2, '0')
                                .uppercase()
                    },
        )
    }

    @Test
    fun bfLayout() {
        val command = BfCommand(
            softLimit = ChannelMap(
                Intensity.of(50),
                Intensity.of(60)
            ),
            frequencyBalance = ChannelMap(128, 129),
            intensityBalance = ChannelMap(130, 131),
        )
        assertEquals(
            listOf(0xBF, 50, 60, 128, 129, 130, 131),
            command.encode().map { it.toInt() and 0xFF })
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            BfCommand(
                softLimit = ChannelMap(Intensity.MIN, Intensity.MIN),
                frequencyBalance = ChannelMap(256, 0),
                intensityBalance = ChannelMap(0, 0),
            )
        }
    }

    @Test
    fun b1Decode() {
        val message = B1Message.decode(byteArrayOf(0xB1.toByte(), 3, 10, 20))
        assertEquals(3, message.sequence.value)
        assertEquals(
            Intensity.of(10),
            message.intensity.a
        )
        assertEquals(
            Intensity.of(20),
            message.intensity.b
        )
        assertFailsWith<ProtocolException.InvalidHeader> {
            B1Message.decode(byteArrayOf(0xB0.toByte(), 3, 10, 20))
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            B1Message.decode(byteArrayOf(0xB1.toByte(), 3, 201.toByte(), 20))
        }
    }

    @Test
    fun batteryDecode() {
        assertEquals(
            BatteryLevel.of(0),
            decodeBattery(byteArrayOf(0))
        )
        assertEquals(
            BatteryLevel.of(100),
            decodeBattery(byteArrayOf(100))
        )
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            decodeBattery(byteArrayOf(101))
        }
        assertFailsWith<ProtocolException.InvalidLength> {
            decodeBattery(byteArrayOf(1, 2))
        }
    }

    @Test
    fun frequencyMapping() {
        val cases = listOf(
            10 to 10,
            100 to 100,
            110 to 102,
            150 to 110,
            600 to 200,
            650 to 205,
            1000 to 240
        )
        cases.forEach { (millis, wire) ->
            assertEquals(
                wire,
                encodeInterval(PulseInterval.of(millis))
            )
        }
        assertEquals(
            105,
            decodeWireFrequency(101).milliseconds
        )
        assertEquals(
            600,
            decodeWireFrequency(200).milliseconds
        )
        assertEquals(
            610,
            decodeWireFrequency(201).milliseconds
        )
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            decodeWireFrequency(9)
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            decodeWireFrequency(241)
        }
    }

}
