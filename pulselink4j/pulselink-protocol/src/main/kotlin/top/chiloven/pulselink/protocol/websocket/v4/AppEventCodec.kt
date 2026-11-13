package top.chiloven.pulselink.protocol.websocket.v4

/** Encodes an [AppEvent] as V4 JSON. */
public fun encodeAppEvent(event: AppEvent): String =
    PulselinkJson.encodeToString(AppEventSerializer, event)

/** Decodes an [AppEvent] from V4 JSON. */
public fun decodeAppEvent(json: String): AppEvent =
    PulselinkJson.decodeFromString(AppEventSerializer, json)
