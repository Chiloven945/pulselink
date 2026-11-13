package top.chiloven.pulselink.ws

import top.chiloven.pulselink.protocol.websocket.v4.ActionType
import kotlinx.coroutines.CompletableDeferred

/** A submitted operation. */
public class OperationHandle internal constructor(
    public val reqId: String,
    public val action: ActionType,
    internal val completion: CompletableDeferred<OperationResult>,
) {

    /** Waits for the operation result. */
    public suspend fun completion(): OperationResult = completion.await()

}
