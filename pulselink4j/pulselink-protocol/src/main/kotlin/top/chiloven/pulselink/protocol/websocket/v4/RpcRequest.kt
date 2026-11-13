package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.json.JsonElement

/** An RPC request. */
public data class RpcRequest(
    public val reqId: String,
    public val method: String,
    public val data: JsonElement? = null,
)
