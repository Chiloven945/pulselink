package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.websocket.v4.internal.requiredString
import top.chiloven.pulselink.protocol.websocket.v4.internal.stringContentOrNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** An application-level message inside a relay `message` frame. */
public sealed interface AppMessage {

    public data class Req(public val request: RpcRequest) : AppMessage
    public data class Resp(public val response: RpcResponse) : AppMessage
    public data class Ev(public val event: AppEvent) : AppMessage

    public companion object {

        public fun toJson(message: AppMessage): JsonObject =
            buildJsonObject {
                when (message) {
                    is Req -> {
                        put("t", "req")
                        put("reqId", message.request.reqId)
                        put("m", message.request.method)
                        message.request.data?.let { put("data", it) }
                    }

                    is Resp -> {
                        put("t", "resp")
                        put("reqId", message.response.reqId)
                        message.response.result?.let { put("result", it) }
                        message.response.error?.let { put("error", it) }
                    }

                    is Ev -> {
                        put("t", "ev")
                        AppEvent.toJson(message.event).forEach { (key, value) -> put(key, value) }
                    }
                }
            }

        public fun fromJson(raw: JsonObject): AppMessage =
            when (val type = raw["t"]?.jsonPrimitive?.stringContentOrNull) {
                "req" -> Req(
                    RpcRequest(
                        reqId = raw.requiredString("reqId"),
                        method = raw.requiredString("m"),
                        data = raw["data"],
                    ),
                )

                "resp" -> Resp(
                    RpcResponse(
                        reqId = raw.requiredString("reqId"),
                        result = raw["result"],
                        error = raw["error"]?.jsonPrimitive?.stringContentOrNull,
                    ),
                )

                "ev" -> Ev(AppEvent.fromJson(raw))

                null -> throw ProtocolException.Json("app message has no t discriminator")

                else -> throw ProtocolException.Json("unknown app message type: $type")
            }

    }

}
