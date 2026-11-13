package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.json.JsonElement

/** An RPC response. */
public data class RpcResponse(
    public val reqId: String,
    public val result: JsonElement? = null,
    public val error: String? = null,
)
