package top.chiloven.pulselink.java.ws;

import top.chiloven.pulselink.java.internal.WsAdapterFactory;

import java.util.concurrent.CompletableFuture;

final class WsFactory {

    private WsFactory() {
    }

    static CompletableFuture<Socket> connect(SocketOptions options) {
        return new WsAdapterFactory().connect(options);
    }

}
