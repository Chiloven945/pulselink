package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.protocol.coyote.v3.Sequence
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

internal class AckTracker {

    private val lock = Mutex()
    private val waiters = mutableMapOf<Int, CompletableDeferred<Unit>>()

    suspend fun register(sequence: Sequence): CompletableDeferred<Unit> {
        val waiter = CompletableDeferred<Unit>()
        lock.withLock { waiters[sequence.value] = waiter }
        return waiter
    }

    suspend fun remove(sequence: Sequence) {
        lock.withLock { waiters.remove(sequence.value) }
    }

    suspend fun complete(sequence: Sequence) {
        val waiter = lock.withLock { waiters.remove(sequence.value) }
        waiter?.complete(Unit)
    }

}
