package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class V4OperationTest {

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
