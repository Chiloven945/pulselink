package top.chiloven.pulselink.protocol.websocket.v4

/** Encodes a [RelayFrame] as V4 JSON. */
public fun encodeRelayFrame(frame: RelayFrame): String =
    PulselinkJson.encodeToString(RelayFrameSerializer, frame)

/** Decodes a [RelayFrame] from V4 JSON. */
public fun decodeRelayFrame(json: String): RelayFrame =
    PulselinkJson.decodeFromString(RelayFrameSerializer, json)
