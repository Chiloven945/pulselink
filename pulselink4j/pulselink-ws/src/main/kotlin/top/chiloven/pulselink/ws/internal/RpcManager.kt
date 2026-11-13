package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.protocol.websocket.v4.AppMessage
import top.chiloven.pulselink.protocol.websocket.v4.RelayFrame
import top.chiloven.pulselink.protocol.websocket.v4.RpcRequest
import top.chiloven.pulselink.ws.WsError
import java.util.concurrent.ConcurrentHashMap
import kotlin.time.Duration
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject

internal class RpcManager(private val ids: RequestIds) {

    private val pending = ConcurrentHashMap<String, PendingRpc>()

    suspend fun request(
        clientId: String,
        method: String,
        data: JsonElement? = null,
        timeout: Duration,
        send: suspend (RelayFrame) -> Unit,
    ): JsonElement {
        val reqId = ids.next()
        val deferred = CompletableDeferred<JsonElement>()
        pending[reqId] = PendingRpc(reqId, method, deferred)
        try {
            send(
                RelayFrame.Message(
                    clientId,
                    AppMessage.toJson(
                        AppMessage.Req(
                            RpcRequest(
                                reqId,
                                method,
                                data
                            ),
                        ),
                    ),
                ),
            )
            return withTimeout(timeout) { deferred.await() }
        } catch (error: Throwable) {
            pending.remove(reqId)
            throw when (error) {
                is TimeoutCancellationException -> WsError.RpcTimeout(reqId)
                is WsError -> error
                is CancellationException -> error
                else -> WsError.Transport(error.message ?: "send failed")
            }
        }
    }

    fun complete(reqId: String, result: JsonElement?, error: String?) {
        pending.remove(reqId)?.let { entry ->
            if (error != null) {
                entry.deferred.completeExceptionally(
                    WsError.Rpc(
                        error,
                        entry.method
                    )
                )
            } else {
                entry.deferred.complete(result ?: JsonObject(emptyMap()))
            }
        }
    }

    fun failAll() {
        pending.values.forEach {
            it.deferred.completeExceptionally(WsError.Disconnected())
        }
        pending.clear()
    }

}
