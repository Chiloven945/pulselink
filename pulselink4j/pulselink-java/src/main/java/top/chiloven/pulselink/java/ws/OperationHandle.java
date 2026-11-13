package top.chiloven.pulselink.java.ws;

import java.util.concurrent.CompletableFuture;

/** A submitted operation. */
public interface OperationHandle {

    String reqId();

    ActionType action();

    CompletableFuture<OperationResult> completion();

}
