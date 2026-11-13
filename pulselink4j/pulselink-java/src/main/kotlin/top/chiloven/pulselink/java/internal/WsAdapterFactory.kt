package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.ws.Socket
import top.chiloven.pulselink.java.ws.SocketOptions
import java.util.concurrent.CompletableFuture
import top.chiloven.pulselink.ws.Socket as KotlinSocket

internal class WsAdapterFactory {

    fun connect(options: SocketOptions): CompletableFuture<Socket> =
        BridgeContext().run {
            future<Socket> {
                val socket = KotlinSocket.connect(options.toKotlin())
                SocketAdapter(this, socket)
            }.whenComplete { _, error ->
                if (error != null) close()
            }
        }

}
