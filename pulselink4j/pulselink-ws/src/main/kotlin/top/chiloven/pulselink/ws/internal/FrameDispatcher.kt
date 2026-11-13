package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.protocol.websocket.v4.AppMessage
import top.chiloven.pulselink.protocol.websocket.v4.RelayFrame
import top.chiloven.pulselink.protocol.websocket.v4.decodeRelayFrame
import top.chiloven.pulselink.ws.SocketEvent
import top.chiloven.pulselink.ws.SocketState
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.serialization.json.jsonObject

internal class FrameDispatcher(
    private val state: MutableStateFlow<SocketState>,
    private val events: MutableSharedFlow<SocketEvent>,
    private val registry: AppRegistry,
    private val rpc: RpcManager,
    private val operations: OperationManager,
    private val onHello: (String) -> Unit,
    private val onCleanup: () -> Unit,
) {

    suspend fun dispatch(text: String) {
        val frame = runCatching { decodeRelayFrame(text) }.getOrElse {
            events.emit(SocketEvent.Error("invalid relay frame: ${it.message}"))
            return
        }
        when (frame) {
            is RelayFrame.Hello -> {
                onHello(frame.clientId)
                state.value = SocketState.WaitingForPeer
                events.emit(SocketEvent.StateChanged(SocketState.WaitingForPeer))
            }

            is RelayFrame.ClientAttached -> {
                registry.attach(frame.clientId)
                state.value = SocketState.Ready
                events.emit(SocketEvent.ClientAttached(frame.clientId))
            }

            is RelayFrame.ClientDisconnected -> {
                registry.detach(frame.clientId)
                events.emit(SocketEvent.ClientDisconnected(frame.clientId))
                if (registry.isEmpty()) {
                    state.value = SocketState.WaitingForPeer
                }
            }

            is RelayFrame.Message -> handleMessage(frame)
            RelayFrame.Heartbeat -> events.emit(SocketEvent.Heartbeat)
            is RelayFrame.Pong -> events.emit(SocketEvent.Pong(frame.ts))
            RelayFrame.IdleTimeout -> {
                events.emit(SocketEvent.IdleTimeout)
                onCleanup()
            }

            is RelayFrame.Error -> events.emit(
                SocketEvent.Error(
                    frame.message ?: frame.code ?: "relay error"
                )
            )

            is RelayFrame.ControllerAttached, is RelayFrame.ControllerDisconnected,
            is RelayFrame.Ping, is RelayFrame.Unknown -> Unit
        }
    }

    private suspend fun handleMessage(frame: RelayFrame.Message) {
        val clientId = frame.clientId ?: return
        val message = runCatching { AppMessage.fromJson(frame.data.jsonObject) }.getOrNull()
            ?: return
        when (message) {
            is AppMessage.Resp -> completeResponse(
                message.response.reqId,
                message.response.result,
                message.response.error
            )

            is AppMessage.Ev -> {
                val app = registry.get(clientId) ?: return
                app.applyEvent(message.event)
            }

            is AppMessage.Req -> Unit
        }
    }

    private fun completeResponse(
        reqId: String,
        result: kotlinx.serialization.json.JsonElement?,
        error: String?,
    ) {
        rpc.complete(reqId, result, error)
        operations.complete(reqId, result, error)
    }

}
