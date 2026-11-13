package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.protocol.websocket.v4.*
import top.chiloven.pulselink.ws.*
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.CompletableDeferred
import kotlinx.serialization.json.JsonElement

internal class OperationManager(
    private val ids: RequestIds,
    private val rpc: RpcManager,
) {

    private val operations = ConcurrentHashMap<String, PendingOperation>()

    suspend fun submit(
        clientId: String,
        operate: DeviceOperate,
        options: OperationOptions,
        send: suspend (RelayFrame) -> Unit,
    ): OperationHandle {
        operate.validate()
        val reqId = ids.next()
        val deferred = CompletableDeferred<OperationResult>()
        operations[reqId] = PendingOperation(deferred)
        try {
            send(
                RelayFrame.Message(
                    clientId,
                    AppMessage.toJson(
                        AppMessage.Req(
                            RpcRequest(
                                reqId,
                                "device.op",
                                PulselinkJson.encodeToJsonElement(
                                    DeviceOperate.serializer(),
                                    operate
                                ),
                            ),
                        ),
                    ),
                ),
            )
        } catch (error: Throwable) {
            operations.remove(reqId)
            throw WsError.Transport(error.message ?: "send failed")
        }
        return OperationHandle(reqId, operate.action, deferred)
    }

    suspend fun clear(
        clientId: String,
        target: ClearTarget,
        timeout: kotlin.time.Duration,
        send: suspend (RelayFrame) -> Unit,
    ) {
        val clear = when (target) {
            ClearTarget.All -> null
            is ClearTarget.Device -> ClearOperate(target.slotId, null)
            is ClearTarget.Channel -> ClearOperate(target.slotId, target.channel.wireValue)
        }
        val data = clear?.let {
            PulselinkJson.encodeToJsonElement(
                ClearOperate.serializer(),
                it
            )
        }
        rpc.request(clientId, "device.op.clear", data, timeout, send)
    }

    fun complete(reqId: String, result: JsonElement?, error: String?) {
        operations.remove(reqId)?.let { entry ->
            if (error != null) {
                entry.deferred.completeExceptionally(
                    WsError.Rpc(error, "device.op")
                )
            } else if (result != null) {
                runCatching {
                    PulselinkJson.decodeFromJsonElement(
                        DeviceOperateResult.serializer(),
                        result
                    )
                }.onSuccess { deviceResult ->
                    entry.deferred.complete(
                        OperationResult(
                            deviceResult.action,
                            deviceResult.reason,
                            deviceResult.slotId,
                            deviceResult.channel?.let(Channel::fromWire),
                        ),
                    )
                }.onFailure {
                    entry.deferred.completeExceptionally(
                        WsError.Json(
                            it.message ?: "invalid result"
                        )
                    )
                }
            }
        }
    }

    fun failAll() {
        operations.values.forEach {
            it.deferred.completeExceptionally(WsError.Disconnected())
        }
        operations.clear()
    }

}
