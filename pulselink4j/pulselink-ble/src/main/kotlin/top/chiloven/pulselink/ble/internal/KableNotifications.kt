package top.chiloven.pulselink.ble.internal

import com.juul.kable.Characteristic
import com.juul.kable.Peripheral
import top.chiloven.pulselink.transport.TransportException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch

internal fun Peripheral.observeMapped(characteristic: Characteristic): Flow<ByteArray> =
    observe(characteristic)
            .catch { error ->
                if (error is CancellationException) throw error
                throw TransportException.NotifyFailed(
                    error.message ?: "notification failed",
                    error
                )
            }
