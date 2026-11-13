package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.AppEvent
import top.chiloven.pulselink.protocol.websocket.v4.decodeAppEvent
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive

class V4EventTest {

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

}
