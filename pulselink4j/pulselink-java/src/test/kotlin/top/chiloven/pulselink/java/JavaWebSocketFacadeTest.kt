package top.chiloven.pulselink.java

import io.ktor.server.application.*
import io.ktor.server.cio.*
import io.ktor.server.engine.*
import io.ktor.server.routing.*
import io.ktor.server.websocket.*
import io.ktor.websocket.*
import top.chiloven.pulselink.java.ws.ClearTarget
import top.chiloven.pulselink.java.ws.Socket
import top.chiloven.pulselink.protocol.websocket.v4.*
import java.util.concurrent.TimeUnit
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject

class JavaWebSocketFacadeTest {

    @Test
    fun connectsAndControlsV4() = runBlocking<Unit> {
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
                                put("devices", JsonArray(emptyList()))
                            }

                            "ping" -> JsonPrimitive(123L)
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
                    .get(10, TimeUnit.SECONDS)

            assertEquals("target", socket.targetId())

            val app = socket.waitForClient()
                    .get(10, TimeUnit.SECONDS)

            assertEquals("app-1", app.clientId())
            assertEquals(
                emptyList(),
                app.requestDevices().get(10, TimeUnit.SECONDS),
            )
            assertEquals(
                123L,
                app.ping().get(10, TimeUnit.SECONDS),
            )

            app.clearOperate(ClearTarget.All())
                    .get(10, TimeUnit.SECONDS)

            socket.shutdown()
                    .get(10, TimeUnit.SECONDS)
        } finally {
            server.stop(0, 0)
        }
    }

}
