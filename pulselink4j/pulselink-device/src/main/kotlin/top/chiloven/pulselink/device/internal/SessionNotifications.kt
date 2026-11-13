package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.transport.GattCharacteristic
import java.util.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class SessionNotifications(
    private val base: BaseSession,
    private val service: UUID,
    private val characteristic: UUID,
    private val decoder: (BaseSession, ByteArray) -> Unit,
) {

    fun subscribe(scope: CoroutineScope): Job =
        scope.launch {
            base.link.subscribe(GattCharacteristic(service, characteristic))
                    .collect { bytes ->
                        decoder(base, bytes)
                    }
        }

}
