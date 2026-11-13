package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.RelayFrame
import top.chiloven.pulselink.protocol.websocket.v4.decodeRelayFrame
import top.chiloven.pulselink.protocol.websocket.v4.encodeRelayFrame
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.serialization.json.jsonPrimitive

class V4FrameTest {

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

}
