package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.ws.OperationResult
import kotlinx.coroutines.CompletableDeferred

internal data class PendingOperation(
    val deferred: CompletableDeferred<OperationResult>,
)
