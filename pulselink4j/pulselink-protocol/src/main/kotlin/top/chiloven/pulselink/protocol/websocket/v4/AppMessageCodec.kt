package top.chiloven.pulselink.protocol.websocket.v4

/** Encodes an [AppMessage] as V4 JSON. */
public fun encodeAppMessage(message: AppMessage): String =
    PulselinkJson.encodeToString(AppMessageSerializer, message)

/** Decodes an [AppMessage] from V4 JSON. */
public fun decodeAppMessage(json: String): AppMessage =
    PulselinkJson.decodeFromString(AppMessageSerializer, json)
