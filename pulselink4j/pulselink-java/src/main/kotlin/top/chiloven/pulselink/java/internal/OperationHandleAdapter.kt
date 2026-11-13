package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.ws.ActionType
import top.chiloven.pulselink.java.ws.OperationHandle
import top.chiloven.pulselink.java.ws.OperationResult
import java.util.concurrent.CompletableFuture
import top.chiloven.pulselink.ws.OperationHandle as KotlinOperationHandle

internal class OperationHandleAdapter(
    private val context: BridgeContext,
    private val handle: KotlinOperationHandle,
) : OperationHandle {

    override fun reqId(): String =
        handle.reqId

    override fun action(): ActionType =
        handle.action.toJava()

    override fun completion(): CompletableFuture<OperationResult> =
        context.future {
            handle.completion().toJava()
        }

}
