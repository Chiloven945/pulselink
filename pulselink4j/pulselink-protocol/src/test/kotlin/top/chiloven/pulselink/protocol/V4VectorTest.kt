package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

class V4VectorTest {

    private val vectors = VectorLoader()

    @Test
    fun v4DeviceOperations() {
        for (case in vectors.cases("websocket-v4/device-op.json")) {
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

    @Test
    fun v4RpcVectors() {
        for (case in vectors.cases("websocket-v4/rpc.json")) {
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

    @Test
    fun v4FramesAndEvents() {
        for (case in vectors.cases("websocket-v4/frames.json")) {
            val input = case.getValue("input").jsonObject
            val frame = decodeRelayFrame(input.toString())
            val variant = case.getValue("expect").jsonObject
                    .getValue("variant").jsonPrimitive.content
            assertEquals(variant, variantName(frame))
        }
        for (case in vectors.cases("websocket-v4/events.json")) {
            val input = case.getValue("input").jsonObject
            val event = decodeAppEvent(input.toString())
            if (case.getValue("expect").jsonObject["valid"]?.jsonPrimitive?.content?.toBoolean() == false) {
                assertFails { event.validate() }
            } else {
                event.validate()
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

    private fun eventName(event: AppEvent): String = when (event) {
        is AppEvent.DevicesSnapshot -> "devices_snapshot"
        is AppEvent.DevicesPatch -> "devices_patch"
        is AppEvent.SlotsPatch -> "slots_patch"
        is AppEvent.CustomAction -> "custom_action"
        is AppEvent.Unknown -> "unknown"
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
