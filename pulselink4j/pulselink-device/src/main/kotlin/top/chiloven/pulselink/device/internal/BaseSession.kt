package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.GattLink
import top.chiloven.pulselink.transport.WriteMode
import java.util.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

internal class BaseSession(
    val link: GattLink,
    notifyService: UUID,
    notifyCharacteristic: UUID,
    decoder: (BaseSession, ByteArray) -> Unit,
    private val initCommand: (suspend () -> Unit)? = null,
) {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val mutableState = MutableStateFlow(DeviceState.empty(ConnectionState.Initializing))
    private val mutableEvents = MutableSharedFlow<LocalEvent>(extraBufferCapacity = 128)
    private val notifications = SessionNotifications(
        this,
        notifyService,
        notifyCharacteristic,
        decoder
    )
    private val battery = SessionBattery(this)
    private var notifyJob: Job? = null
    private var batteryJob: Job? = null
    private var closed = false

    val state: StateFlow<DeviceState> get() = mutableState.asStateFlow()
    val events: SharedFlow<LocalEvent> get() = mutableEvents.asSharedFlow()

    suspend fun initialize() {
        notifyJob = notifications.subscribe(scope)
        batteryJob = battery.subscribe(scope)
        battery.readLevel()?.let(::updateBattery)
        initCommand?.invoke()
        mutableState.value = mutableState.value.copy(connection = ConnectionState.Ready).touch()
    }

    suspend fun writeRaw(characteristic: UUID, bytes: ByteArray) {
        link.write(
            GattCharacteristic(GattIds.CONTROL_SERVICE, characteristic),
            bytes,
            WriteMode.Auto
        )
    }

    fun updateBattery(level: BatteryLevel) {
        mutableState.value = mutableState.value.copy(
            battery = Observed.Known(BatteryState(level)),
            revision = mutableState.value.revision + 1,
        )
        mutableEvents.tryEmit(LocalEvent.BatteryChanged(BatteryState(level)))
    }

    fun emit(event: LocalEvent) {
        mutableEvents.tryEmit(event)
    }

    fun updateIntensity(channel: Channel, intensity: Intensity) {
        val current = mutableState.value.channels[channel]
                .copy(intensity = Observed.Known(intensity))
        val channels = when (channel) {
            Channel.A -> ChannelMap(current, mutableState.value.channels.b)
            Channel.B -> ChannelMap(mutableState.value.channels.a, current)
        }
        mutableState.value = mutableState.value.copy(
            channels = channels,
            revision = mutableState.value.revision + 1
        )
        mutableEvents.tryEmit(LocalEvent.IntensityChanged(channel, intensity))
    }

    suspend fun shutdown() {
        if (closed) return
        closed = true
        try {
            writeRaw(GattIds.CONTROL_WRITE, zeroForKind())
        } catch (error: CancellationException) {
            throw error
        } catch (_: Exception) {
            // Best-effort safety write; shutdown must continue.
        }
        notifyJob?.cancel()
        batteryJob?.cancel()
        try {
            link.disconnect()
        } catch (error: CancellationException) {
            throw error
        } catch (_: Exception) {
            // Best-effort disconnect; the scope is cancelled below.
        }
        mutableState.value = mutableState.value
                .copy(connection = ConnectionState.Disconnected)
                .touch()
        mutableEvents
                .emit(LocalEvent.Closed)
        scope.cancel()
    }

    fun close() {
        if (closed) return
        closed = true
        notifyJob?.cancel()
        batteryJob?.cancel()
        scope.cancel()
        mutableState.value = mutableState.value
                .copy(connection = ConnectionState.Disconnected)
                .touch()
    }

    private fun zeroForKind(): ByteArray =
        ByteArray(0)

}
