package top.chiloven.pulselink.ble.internal

import top.chiloven.pulselink.transport.TransportException
import kotlin.time.Duration
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.withTimeout

internal suspend fun <T> withOperationTimeout(
    timeout: Duration,
    block: suspend () -> T,
): T =
    try {
        withTimeout(timeout) { block() }
    } catch (_: TimeoutCancellationException) {
        throw TransportException.Timeout()
    }
