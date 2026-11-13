package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.ws.AppClient
import top.chiloven.pulselink.java.ws.Socket
import top.chiloven.pulselink.java.ws.SocketEvent
import top.chiloven.pulselink.java.ws.SocketState
import java.util.*
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.ws.Socket as KotlinSocket

internal class SocketAdapter(
    private val context: BridgeContext,
    private val socket: KotlinSocket,
) : Socket {

    override fun targetId(): String =
        socket.targetId

    override fun state(): SocketState =
        socket.state.value.toJava()

    override fun events(): Flow.Publisher<SocketEvent> =
        context.publisher(
            socket.events.map { it.toJava() },
            conflate = false
        )

    override fun waitForClient(): CompletableFuture<AppClient> =
        context.future {
            AppClientAdapter(context, socket.waitForClient())
        }

    override fun clients(): List<AppClient> =
        socket.clients().map { AppClientAdapter(context, it) }

    override fun client(clientId: String): Optional<AppClient> =
        socket.client(clientId)
                ?.let {
                    Optional.of(AppClientAdapter(context, it))
                }
            ?: Optional.empty()

    override fun shutdown(): CompletableFuture<Void> =
        context
                .futureVoid { socket.shutdown() }
                .whenComplete { _, _ -> context.close() }

    override fun close() {
        socket.close()
        context.close()
    }

}
