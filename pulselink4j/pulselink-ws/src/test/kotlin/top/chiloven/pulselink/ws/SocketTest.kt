package top.chiloven.pulselink.ws

import io.ktor.server.application.*
import io.ktor.server.cio.*
import io.ktor.server.engine.*
import io.ktor.server.routing.*
import io.ktor.server.websocket.*
import io.ktor.websocket.*
import top.chiloven.pulselink.protocol.websocket.v4.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject

class SocketTest {

    @Test
    fun handshakeDevicesAndPing() = runBlocking {
        val server = embeddedServer(CIO, port = 0) {
            install(WebSockets)
            routing {
                webSocket("/v4") {
                    send(Frame.Text(encodeRelayFrame(RelayFrame.Hello("target"))))
                    send(Frame.Text(encodeRelayFrame(RelayFrame.ClientAttached("app-1"))))
                    for (frame in incoming) {
                        if (frame !is Frame.Text) continue
                        val relay = decodeRelayFrame(frame.readText())
                        if (relay !is RelayFrame.Message) continue
                        val app = AppMessage.fromJson(relay.data as JsonObject)
                        if (app !is AppMessage.Req) continue
                        val result = when (app.request.method) {
                            "devices.get" -> buildJsonObject {
                                put(
                                    "devices",
                                    kotlinx.serialization.json.JsonArray(emptyList())
                                )
                            }

                            "ping" -> kotlinx.serialization.json.JsonPrimitive(123L)
                            "device.op.clear" -> JsonObject(emptyMap())
                            else -> JsonObject(emptyMap())
                        }
                        send(
                            Frame.Text(
                                encodeRelayFrame(
                                    RelayFrame.Message(
                                        relay.clientId,
                                        AppMessage.toJson(
                                            AppMessage.Resp(
                                                RpcResponse(
                                                    app.request.reqId,
                                                    result,
                                                    null,
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        )
                    }
                }
            }
        }.start(wait = false)

        try {
            val port = server.engine.resolvedConnectors().first().port
            val socket = Socket.connect("ws://127.0.0.1:$port/v4")
            assertEquals("target", socket.targetId)
            val client = socket.waitForClient()
            assertEquals("app-1", client.clientId)
            assertEquals(emptyList(), client.requestDevices())
            assertEquals(123L, client.ping())
            client.clearOperate(ClearTarget.All)
            assertTrue(socket.clients().isNotEmpty())
            socket.shutdown()
        } finally {
            server.stop(0, 0)
        }
    }

}
