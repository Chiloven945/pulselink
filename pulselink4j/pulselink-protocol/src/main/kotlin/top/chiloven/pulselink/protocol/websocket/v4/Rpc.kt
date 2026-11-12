package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.descriptors.buildClassSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.*

/** An RPC request. */
public data class RpcRequest(
    public val reqId: String,
    public val method: String,
    public val data: JsonElement? = null,
)

/** An RPC response. */
public data class RpcResponse(
    public val reqId: String,
    public val result: JsonElement? = null,
    public val error: String? = null,
)

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
            when (val type = raw["t"]?.jsonPrimitive?.contentOrNull) {
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
                        error = raw["error"]?.jsonPrimitive?.contentOrNull,
                    ),
                )

                "ev" -> Ev(AppEvent.fromJson(raw))

                null -> throw ProtocolException.Json("app message has no t discriminator")

                else -> throw ProtocolException.Json("unknown app message type: $type")
            }

    }

}

private fun JsonObject.requiredString(key: String): String =
    this[key]?.jsonPrimitive?.contentOrNull
        ?: throw ProtocolException.Json("missing string field: $key")

private val JsonPrimitive.contentOrNull: String?
    get() = if (isString) content else null

/** Serializer for [AppMessage]. */
public object AppMessageSerializer : KSerializer<AppMessage> {

    override val descriptor: SerialDescriptor = buildClassSerialDescriptor("AppMessage")

    override fun serialize(encoder: Encoder, value: AppMessage) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("AppMessage requires a JSON encoder")
        jsonEncoder.encodeJsonElement(AppMessage.toJson(value))
    }

    override fun deserialize(decoder: Decoder): AppMessage {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("AppMessage requires a JSON decoder")
        return AppMessage.fromJson(jsonDecoder.decodeJsonElement().jsonObject)
    }

}

/** Encodes an [AppMessage] as V4 JSON. */
public fun encodeAppMessage(message: AppMessage): String =
    PulselinkJson.encodeToString(AppMessageSerializer, message)

/** Decodes an [AppMessage] from V4 JSON. */
public fun decodeAppMessage(json: String): AppMessage =
    PulselinkJson.decodeFromString(AppMessageSerializer, json)
