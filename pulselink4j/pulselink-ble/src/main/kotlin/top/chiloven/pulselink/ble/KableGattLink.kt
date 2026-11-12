package top.chiloven.pulselink.ble

import com.juul.kable.Peripheral
import com.juul.kable.State
import top.chiloven.pulselink.transport.*
import kotlin.time.Duration
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeout

internal class KableGattLink(
    override val deviceId: TransportDeviceId,
    private val peripheral: Peripheral,
    private val characteristicMap: Map<GattCharacteristic, CharacteristicInfo>,
    private val mtuValue: Int?,
    private val operationTimeout: Duration,
) : GattLink {

    private val writeMutex = Mutex()

    override val isConnected: Boolean
        get() = peripheral.state.value is State.Connected

    override fun mtu(): Int? =
        mtuValue

    override fun characteristics(): Map<GattCharacteristic, CharacteristicInfo> =
        characteristicMap

    override fun characteristic(id: GattCharacteristic): CharacteristicInfo? =
        characteristicMap[id]

    override suspend fun write(
        characteristic: GattCharacteristic,
        data: ByteArray,
        mode: WriteMode,
    ) {
        writeMutex.withLock {
            ensureConnected()
            val info = characteristic(characteristic)
                ?: throw TransportException.GattFailed("characteristic not found: $characteristic")
            val writeType = selectWriteType(info, mode)
            try {
                withTimeout(operationTimeout) {
                    peripheral.write(info.id.toKable(), data, writeType)
                }
            } catch (_: TimeoutCancellationException) {
                throw TransportException.Timeout()
            } catch (error: CancellationException) {
                throw error
            } catch (error: Exception) {
                throw mapGattError(error)
            }
        }
    }

    override suspend fun read(characteristic: GattCharacteristic): ByteArray {
        ensureConnected()
        val info = characteristic(characteristic)
            ?: throw TransportException.GattFailed("characteristic not found: $characteristic")
        if (!info.readable) {
            throw TransportException.GattFailed("characteristic does not support read")
        }

        return try {
            withTimeout(operationTimeout) {
                peripheral.read(info.id.toKable())
            }
        } catch (_: TimeoutCancellationException) {
            throw TransportException.Timeout()
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            throw mapGattError(error)
        }
    }

    override fun subscribe(characteristic: GattCharacteristic): Flow<ByteArray> {
        val info = characteristic(characteristic)
            ?: throw TransportException.NotifyFailed("characteristic not found: $characteristic")
        if (!info.notifiable && !info.indicatable) {
            throw TransportException.NotifyFailed("characteristic does not support notify or indicate")
        }

        return peripheral.observe(info.id.toKable())
                .catch { error ->
                    if (error is CancellationException) throw error
                    throw TransportException.NotifyFailed(
                        error.message ?: "notification failed",
                        error
                    )
                }
    }

    override suspend fun disconnect() {
        try {
            withTimeout(operationTimeout) {
                peripheral.disconnect()
            }
        } catch (_: TimeoutCancellationException) {
            throw TransportException.Timeout()
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            throw TransportException.GattFailed(error.message ?: "disconnect failed", error)
        }
    }

    private fun ensureConnected() {
        if (!isConnected) throw TransportException.Disconnected()
    }

    private fun mapGattError(error: Exception): TransportException =
        when (error) {
            is TransportException -> error
            else -> TransportException.GattFailed(
                error.message ?: error::class.simpleName ?: "GATT error", error
            )
        }

}
