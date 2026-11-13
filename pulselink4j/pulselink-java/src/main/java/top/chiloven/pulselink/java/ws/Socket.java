package top.chiloven.pulselink.java.ws;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Flow;

/** A DG-LAB V4 controller socket. */
public interface Socket extends AutoCloseable {

    static CompletableFuture<Socket> connect(String url) {
        return WsFactory.connect(SocketOptions.defaults(url));
    }

    static CompletableFuture<Socket> connect(SocketOptions options) {
        return WsFactory.connect(options);
    }

    String targetId();

    SocketState state();

    Flow.Publisher<SocketEvent> events();

    CompletableFuture<AppClient> waitForClient();

    List<AppClient> clients();

    Optional<AppClient> client(String clientId);

    CompletableFuture<Void> shutdown();

    @Override
    void close();

}
