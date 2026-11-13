package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.ws.AppClient
import top.chiloven.pulselink.ws.Socket
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.CompletableDeferred

internal class AppRegistry(private val socket: Socket) {

    private val clients = ConcurrentHashMap<String, AppClient>()
    private val firstClient = CompletableDeferred<AppClient>()

    fun attach(clientId: String): AppClient {
        val app = AppClient(clientId, socket)
        clients[clientId] = app
        firstClient.complete(app)
        return app
    }

    fun detach(clientId: String) {
        clients.remove(clientId)
    }

    fun first(): AppClient? =
        clients.values.firstOrNull()

    suspend fun awaitFirst(): AppClient =
        firstClient.await()

    fun all(): List<AppClient> =
        clients.values.toList()

    fun get(clientId: String): AppClient? =
        clients[clientId]

    fun isEmpty(): Boolean =
        clients.isEmpty()

    fun disconnectAll() {
        clients.values.forEach { it.disconnected() }
        clients.clear()
    }

}
