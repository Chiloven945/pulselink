package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.ConnectionState
import top.chiloven.pulselink.runtime.CoyoteV3Event
import top.chiloven.pulselink.runtime.RuntimeException
import top.chiloven.pulselink.transport.ReconnectTarget
import kotlinx.coroutines.delay

internal class ReconnectController(private val core: SessionCore) {

    suspend fun reconnect() {
        core.checkOpen()
        core.updateState {
            it.copy(
                common = it.common.copy(
                    connection = ConnectionState.Reconnecting
                ).touch()
            )
        }
        var delayMs = core.config.reconnect.initialBackoff
        for (attempt in 1..core.config.reconnect.maxAttempts) {
            core.emit(CoyoteV3Event.Reconnecting(attempt, delayMs))
            try {
                val newLink = core.backend.reconnect(
                    ReconnectTarget(core.link.deviceId, core.fallbackFilter)
                )
                core.link = newLink
                core.initialize()
                core.emit(CoyoteV3Event.Reconnected)
                return
            } catch (error: Throwable) {
                if (attempt == core.config.reconnect.maxAttempts) {
                    core.emit(CoyoteV3Event.ReconnectFailed(attempt, error))
                    throw RuntimeException.ReconnectFailed(attempt)
                }
                delay(delayMs)
                delayMs = (delayMs * 2).coerceAtMost(core.config.reconnect.maxBackoff)
            }
        }
    }

}
