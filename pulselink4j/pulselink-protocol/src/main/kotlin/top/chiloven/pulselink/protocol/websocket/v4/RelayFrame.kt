package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.websocket.v4.internal.requiredString
import top.chiloven.pulselink.protocol.websocket.v4.internal.stringContentOrNull
import kotlinx.serialization.json.*

/** A frame exchanged with the V4 WebSocket relay. */
public sealed interface RelayFrame {

    public data class Hello(public val clientId: String) : RelayFrame

    public data class ClientAttached(public val clientId: String) : RelayFrame

    public data class ClientDisconnected(public val clientId: String) : RelayFrame

    public data class ControllerAttached(public val clientId: String) : RelayFrame

    public data class ControllerDisconnected(public val clientId: String) : RelayFrame

    public data object Heartbeat : RelayFrame

    public data class Ping(public val ts: Long? = null) : RelayFrame

    public data class Pong(public val ts: Long? = null) : RelayFrame

    public data class Message(
        public val clientId: String?,
        public val data: JsonElement
    ) : RelayFrame

    public data object IdleTimeout : RelayFrame

    public data class Error(
        public val code: String?,
        public val message: String?
    ) : RelayFrame

    public data class Unknown(public val raw: JsonObject) : RelayFrame

    public companion object {

        public fun toJson(frame: RelayFrame): JsonObject =
            buildJsonObject {
                when (frame) {
                    is Hello -> {
                        put("type", "hello")
                        put("clientId", frame.clientId)
                    }

                    is ClientAttached -> {
                        put("type", "client_attached")
                        put("clientId", frame.clientId)
                    }

                    is ClientDisconnected -> {
                        put("type", "client_disconnected")
                        put("clientId", frame.clientId)
                    }

                    is ControllerAttached -> {
                        put("type", "controller_attached")
                        put("clientId", frame.clientId)
                    }

                    is ControllerDisconnected -> {
                        put("type", "controller_disconnected")
                        put("clientId", frame.clientId)
                    }

                    Heartbeat ->
                        put("type", "heartbeat")

                    is Ping -> {
                        put("type", "ping")
                        frame.ts?.let { put("ts", it) }
                    }

                    is Pong -> {
                        put("type", "pong")
                        frame.ts?.let { put("ts", it) }
                    }

                    is Message -> {
                        put("type", "message")
                        frame.clientId?.let { put("clientId", it) }
                        put("data", frame.data)
                    }

                    IdleTimeout ->
                        put("type", "idle_timeout")

                    is Error -> {
                        put("type", "error")
                        frame.code?.let { put("code", it) }
                        frame.message?.let { put("message", it) }
                    }

                    is Unknown -> return frame.raw
                }
            }

        public fun fromJson(raw: JsonObject): RelayFrame =
            when (val type = raw["type"]?.jsonPrimitive?.stringContentOrNull) {
                "hello" -> Hello(raw.requiredString("clientId"))

                "client_attached" -> ClientAttached(raw.requiredString("clientId"))

                "client_disconnected" -> ClientDisconnected(raw.requiredString("clientId"))

                "controller_attached" -> ControllerAttached(raw.requiredString("clientId"))

                "controller_disconnected" -> ControllerDisconnected(raw.requiredString("clientId"))

                "heartbeat" -> Heartbeat

                "ping" -> Ping(raw["ts"]?.jsonPrimitive?.longOrNull)

                "pong" -> Pong(raw["ts"]?.jsonPrimitive?.longOrNull)

                "message" -> Message(
                    clientId = raw["clientId"]?.jsonPrimitive?.stringContentOrNull,
                    data = raw["data"] ?: throw ProtocolException.Json("message requires data"),
                )

                "idle_timeout" -> IdleTimeout

                "error" -> Error(
                    code = raw["code"]?.jsonPrimitive?.stringContentOrNull,
                    message = raw["message"]?.jsonPrimitive?.stringContentOrNull,
                )

                null -> Unknown(raw)

                else -> Unknown(raw)
            }

    }

}
