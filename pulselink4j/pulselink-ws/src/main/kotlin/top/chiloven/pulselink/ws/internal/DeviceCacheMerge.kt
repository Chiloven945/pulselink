package top.chiloven.pulselink.ws.internal

import kotlinx.serialization.json.JsonObject

/** Recursively merges [patch] into [target], preserving nested objects. */
internal fun mergeJson(target: JsonObject, patch: JsonObject): JsonObject {
    val result = target.toMutableMap()
    patch.forEach { (key, value) ->
        val existing = result[key]
        result[key] = if (existing is JsonObject && value is JsonObject) {
            mergeJson(existing, value)
        } else {
            value
        }
    }
    return JsonObject(result)
}
