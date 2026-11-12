package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive

class V4Test {

    @Test
    fun relayFramesRoundTrip() {
        val hello = decodeRelayFrame(
            /* language=JSON */
            """
            {
              "type": "hello",
              "clientId": "controller-1"
            }
            """.trimIndent()
        )
        assertEquals(
            RelayFrame.Hello("controller-1"),
            hello
        )
        assertEquals(
            hello,
            decodeRelayFrame(encodeRelayFrame(hello))
        )

        val message = decodeRelayFrame(
            /* language=JSON */
            """
            {
              "type": "message",
              "clientId": "app-1",
              "data": {
                "t": "req",
                "reqId": "req-1",
                "m": "devices.get"
              }
            }
            """.trimIndent(),
        )
        assertIs<RelayFrame.Message>(message)
        assertEquals(
            "app-1",
            message.clientId
        )

        val unknown = decodeRelayFrame(
            /* language=JSON */
            """
            {
              "type": "future_frame",
              "x": 1
            }
            """.trimIndent()
        )
        assertIs<RelayFrame.Unknown>(unknown)
        assertEquals(
            "future_frame",
            unknown.raw["type"]?.jsonPrimitive?.content
        )
        assertEquals(
            unknown,
            decodeRelayFrame(encodeRelayFrame(unknown))
        )
    }

    @Test
    fun rpcEventsAndUnknown() {
        val snapshot = decodeAppMessage(
            /* language=JSON */
            """
            {
              "t": "ev",
              "ev": "devices.snapshot",
              "devices": [
                {
                  "slotId": "a",
                  "name": "A",
                  "type": "COYOTE_030"
                }
              ]
            }
            """.trimIndent(),
        )
        assertIs<AppMessage.Ev>(snapshot)
        val event = snapshot.event
        assertIs<AppEvent.DevicesSnapshot>(event)
        assertEquals(
            "a",
            event.devices.first().slotId
        )

        val unknown = decodeAppMessage(
            /* language=JSON */
            """
            {
              "t": "ev",
              "ev": "future.event",
              "data": 1
            }
            """.trimIndent()
        )
        assertIs<AppEvent.Unknown>((unknown as AppMessage.Ev).event)
    }

    @Test
    fun eventsPreserveOpenFieldsAndValidateActions() {
        val event = decodeAppEvent(
            /* language=JSON */
            """
            {
              "ev": "devices.snapshot",
              "devices": [
                {
                  "id": 0,
                  "slotId": "a",
                  "name": "A",
                  "type": "COYOTE_030",
                  "props": {
                    "future": {
                      "x": 1
                    }
                  },
                  "slotState": {
                    "hasDevice": true
                  }
                }
              ]
            }
            """.trimIndent(),
        )
        val device = (event as AppEvent.DevicesSnapshot).devices.single()
        assertEquals(
            1,
            device.props["future"]
                    ?.let { it as? JsonObject }
                    ?.get("x")
                    ?.jsonPrimitive
                    ?.content
                    ?.toInt()
        )
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            decodeAppEvent(
                /* language=JSON */
                """
                {
                  "ev": "custom.action",
                  "action": 10
                }
                """.trimIndent()
            ).validate()
        }
    }

    @Test
    fun deviceOperateVectors() {
        val append = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {
              "s": "slot-a",
              "t": 0,
              "c": 0,
              "d": 1000,
              "v": [
                "0A0A0A0A00000000"
              ]
            }
            """.trimIndent(),
        )
        assertEquals(ActionType.AppendPulseData, append.action)
        append.validate()
        assertEquals(
            PulseFrameWire.Hex("0A0A0A0A00000000"),
            (append.value as OperateValue.PulseFrames).frames.single(),
        )

        val appendV2 = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {
              "s": "slot-a",
              "t": 0,
              "c": 1,
              "v": [
                [
                  1,
                  2,
                  3
                ]
              ],
              "ver": 2
            }
            """.trimIndent(),
        )
        appendV2.validate()

        val add = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {
              "s": "slot-a",
              "t": 3,
              "c": 1,
              "v": -7
            }
            """.trimIndent(),
        )
        add.validate()

        val temp = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {
              "s": "slot-a",
              "t": 4,
              "c": 0,
              "d": 3000,
              "v": 30
            }
            """.trimIndent(),
        )
        temp.validate()

        val reset = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {
              "s": "slot-a",
              "t": 7,
              "c": 1,
              "v": 0
            }
            """.trimIndent(),
        )
        reset.validate()

        val invalid = PulselinkJson.decodeFromString(
            DeviceOperate.serializer(),
            """
            {"s":"slot-a","t":7,"c":0,"v":5}
            """.trimIndent(),
        )
        assertFailsWith<ProtocolException.InvalidParameters> { invalid.validate() }
    }

    @Test
    fun clearValidation() {
        val all = ClearOperate()
        all.validate()
        val channel = ClearOperate(
            slotId = "a",
            channel = 0
        )
        channel.validate()
        assertFailsWith<ProtocolException.InvalidParameters> {
            ClearOperate(channel = 0).validate()
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            ClearOperate(
                slotId = "a",
                channel = 2
            ).validate()
        }
    }

}
