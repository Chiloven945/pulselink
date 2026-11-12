package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.pattern.PulseIntensity
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.pattern.PulseSample
import top.chiloven.pulselink.protocol.coyote.v3.*
import top.chiloven.pulselink.protocol.websocket.v4.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlinx.serialization.json.*

class VectorTest {

    private fun cases(path: String): List<JsonObject> {
        val text = checkNotNull(javaClass.getResource("/$path")) {
            "missing resource $path"
        }.readText()
        return PulselinkJson.parseToJsonElement(text)
                .jsonObject.getValue("cases")
                .jsonArray.map { it.jsonObject }
    }

    private fun hex(bytes: ByteArray): String =
        bytes.joinToString("") {
            (it.toInt() and 0xFF)
                    .toString(16)
                    .padStart(2, '0')
                    .uppercase()
        }

    private fun unhex(value: String): ByteArray =
        ByteArray(value.length / 2) {
            value.substring(it * 2, it * 2 + 2)
                    .toInt(16)
                    .toByte()
        }

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
        for (case in cases("coyote-v3/b0.json")) {
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
                hex(command.encode()),
                case.getValue("id").jsonPrimitive.content
            )
        }
    }

    @Test
    fun v3BfVectors() {
        for (case in cases("coyote-v3/bf.json")) {
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
                hex(command.encode()),
                case.getValue("id").jsonPrimitive.content
            )
        }
    }

    @Test
    fun v3B1Vectors() {
        for (case in cases("coyote-v3/b1.json")) {
            val input = case.getValue("input").jsonObject.getValue("hex").jsonPrimitive.content
            val expect = case.getValue("expect").jsonObject
            if (expect["error"]?.jsonPrimitive?.content?.toBoolean() == true) {
                assertFails { B1Message.decode(unhex(input)) }
            } else {
                val message = B1Message.decode(unhex(input))
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
        for (case in cases("coyote-v3/frequency.json")) {
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

    @Test
    fun v4DeviceOperations() {
        for (case in cases("websocket-v4/device-op.json")) {
            val kind = case.getValue("kind").jsonPrimitive.content
            val input = case.getValue("input").jsonObject
            val expect = case.getValue("expect").jsonObject
            val valid = expect["valid"]?.jsonPrimitive?.content?.toBoolean()
            when (kind) {
                "device_operate" -> {
                    val operation = PulselinkJson.decodeFromString(
                        DeviceOperate.serializer(),
                        input.toString()
                    )
                    if (valid == true) {
                        operation.validate()
                        val action = expect.getValue("action").jsonPrimitive.content
                        assertEquals(action, actionName(operation.action))
                    } else {
                        assertFails { operation.validate() }
                    }
                }

                "clear_operate" -> {
                    val clear = PulselinkJson.decodeFromString(
                        ClearOperate.serializer(),
                        input.toString()
                    )
                    if (valid == true) clear.validate() else assertFails { clear.validate() }
                }
            }
        }
    }

    private fun actionName(action: ActionType): String =
        when (action) {
            ActionType.AppendPulseData -> "append_pulse_data"
            ActionType.AddIntensity -> "add_intensity"
            ActionType.SetTempIntensity -> "set_temp_intensity"
            ActionType.SetIntensity -> "set_intensity"
        }

    @Test
    fun v4RpcVectors() {
        for (case in cases("websocket-v4/rpc.json")) {
            val input = case.getValue("input").jsonObject
            val message = decodeAppMessage(input.toString())
            val expect = case.getValue("expect").jsonObject
            val variant = expect.getValue("variant").jsonPrimitive.content
            when (message) {
                is AppMessage.Req -> assertEquals(
                    "req",
                    variant
                )

                is AppMessage.Resp -> assertEquals(
                    "resp",
                    variant
                )

                is AppMessage.Ev -> {
                    assertEquals("ev", variant)
                    val event = expect["event"]?.jsonPrimitive?.content
                    if (event != null) assertEquals(event, eventName(message.event))
                }
            }
        }
    }

    private fun eventName(event: AppEvent): String = when (event) {
        is AppEvent.DevicesSnapshot -> "devices_snapshot"
        is AppEvent.DevicesPatch -> "devices_patch"
        is AppEvent.SlotsPatch -> "slots_patch"
        is AppEvent.CustomAction -> "custom_action"
        is AppEvent.Unknown -> "unknown"
    }

    @Test
    fun v4FramesAndEvents() {
        for (case in cases("websocket-v4/frames.json")) {
            val input = case.getValue("input").jsonObject
            val frame = decodeRelayFrame(input.toString())
            val variant = case.getValue("expect").jsonObject
                    .getValue("variant").jsonPrimitive.content
            assertEquals(variant, variantName(frame))
        }
        for (case in cases("websocket-v4/events.json")) {
            val input = case.getValue("input").jsonObject
            val event = decodeAppEvent(input.toString())
            if (case.getValue("expect").jsonObject["valid"]?.jsonPrimitive?.content?.toBoolean() == false) {
                assertFails { event.validate() }
            } else {
                event.validate()
            }
        }
    }

    private fun variantName(frame: RelayFrame): String =
        when (frame) {
            is RelayFrame.Hello -> "hello"
            is RelayFrame.ClientAttached -> "client_attached"
            is RelayFrame.ClientDisconnected -> "client_disconnected"
            is RelayFrame.ControllerAttached -> "controller_attached"
            is RelayFrame.ControllerDisconnected -> "controller_disconnected"
            RelayFrame.Heartbeat -> "heartbeat"
            is RelayFrame.Ping -> "ping"
            is RelayFrame.Pong -> "pong"
            is RelayFrame.Message -> "message"
            RelayFrame.IdleTimeout -> "idle_timeout"
            is RelayFrame.Error -> "error"
            is RelayFrame.Unknown -> "unknown"
        }

}
