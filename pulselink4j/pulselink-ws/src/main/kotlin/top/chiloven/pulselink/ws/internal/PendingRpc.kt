package top.chiloven.pulselink.ws.internal

import kotlinx.coroutines.CompletableDeferred
import kotlinx.serialization.json.JsonElement

internal data class PendingRpc(
    val reqId: String,
    val method: String,
    val deferred: CompletableDeferred<JsonElement>,
)
