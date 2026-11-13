package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.json.Json

/** The JSON configuration used by all V4 protocol types. */
public val PulselinkJson: Json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    encodeDefaults = true
    isLenient = false
    allowSpecialFloatingPointValues = false
}
