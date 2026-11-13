package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.AppEvent
import top.chiloven.pulselink.protocol.websocket.v4.AppMessage
import top.chiloven.pulselink.protocol.websocket.v4.decodeAppMessage
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class V4RpcTest {

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

}
