package top.chiloven.pulselink.protocol.websocket.v4.internal

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.websocket.v4.PulselinkJson
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonPrimitive

internal fun JsonObject.requiredString(key: String): String =
    this[key]?.jsonPrimitive?.stringContentOrNull
        ?: throw ProtocolException.Json("missing string field: $key")

internal val JsonPrimitive.stringContentOrNull: String?
    get() = if (isString) content else null

internal inline fun <reified T> JsonObject.valueAs(key: String): T? =
    this[key]?.let { PulselinkJson.decodeFromJsonElement(it) }
