package top.chiloven.pulselink.ws.internal

import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import top.chiloven.pulselink.protocol.websocket.v4.RelayFrame
import top.chiloven.pulselink.protocol.websocket.v4.encodeRelayFrame
import top.chiloven.pulselink.ws.WsError
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

internal class SocketWriter(
    private val session: DefaultClientWebSocketSession,
    private val isClosed: () -> Boolean,
) {

    private val mutex = Mutex()

    suspend fun send(frame: RelayFrame) {
        if (isClosed()) throw WsError.Closed()
        val text = encodeRelayFrame(frame)
        mutex.withLock { session.send(Frame.Text(text)) }
    }

}
