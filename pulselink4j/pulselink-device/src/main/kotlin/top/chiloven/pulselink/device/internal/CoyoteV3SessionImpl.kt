package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.pattern.Pattern
import top.chiloven.pulselink.runtime.CoyoteV3Event
import top.chiloven.pulselink.runtime.CoyoteV3Session
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

internal class CoyoteV3SessionImpl(
    private val session: CoyoteV3Session,
) {

    private val sessionScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    val kind: DeviceKind
        get() = DeviceKind.CoyoteV3
    val state: StateFlow<DeviceState>
        field = MutableStateFlow(session.state.value.common)
    val events: SharedFlow<LocalEvent>
        field = MutableSharedFlow<LocalEvent>(extraBufferCapacity = 64)

    val runtime: CoyoteV3Session
        get() = session

    init {
        sessionScope.launch {
            session.state.collect { state.value = it.common }
        }
        sessionScope.launch {
            session.events.collect {
                if (it is CoyoteV3Event.Device) {
                    events.emit(LocalEvent.SensorData)
                }
            }
        }
    }

    suspend fun setIntensity(channel: Channel, intensity: Intensity) {
        session.setIntensity(channel, intensity)
    }

    suspend fun adjustIntensity(
        channel: Channel,
        delta: IntensityDelta
    ) {
        session.adjustIntensity(channel, delta)
    }

    suspend fun play(pattern: Pattern) {
        session.play(pattern)
    }

    suspend fun stop() {
        session.stop()
    }

    suspend fun reconnect(): Unit = session.reconnect()

    suspend fun shutdown() {
        session.close()
        sessionScope.cancel()
    }

    fun close() {
        session.close()
        sessionScope.cancel()
    }

}
