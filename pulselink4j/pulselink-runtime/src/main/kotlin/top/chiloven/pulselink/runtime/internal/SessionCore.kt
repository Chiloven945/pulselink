package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.protocol.coyote.v3.*
import top.chiloven.pulselink.runtime.*
import top.chiloven.pulselink.transport.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.sync.Mutex

internal class SessionCore(
    val backend: Backend,
    var link: GattLink,
    val info: DeviceInfo,
    val config: CoyoteV3Config,
    val fallbackFilter: ScanFilter,
) {

    val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val mutableState = MutableStateFlow(
        CoyoteV3State(
            common = DeviceState.empty(ConnectionState.Connecting),
            settings = Observed.Unknown,
        ),
    )
    private val mutableEvents = MutableSharedFlow<CoyoteV3Event>(extraBufferCapacity = 256)

    val state: StateFlow<CoyoteV3State> get() = mutableState.asStateFlow()
    val events: SharedFlow<CoyoteV3Event> get() = mutableEvents.asSharedFlow()

    val writeMutex = Mutex()
    val playbackLock = Mutex()

    var notifyJob: Job? = null
    var batteryJob: Job? = null
    var playJob: Job? = null
    var playId: PatternId? = null
    var playHandle: PlayHandle? = null
    var closed: Boolean = false
    var nextPatternId: Long = 0L

    val ackTracker = AckTracker()
    val b0Writer = B0Writer(this)
    val notificationRouter = NotificationRouter(this)
    val initialization = Initialization(this)
    val intensityController = IntensityController(this)
    val playbackController = PlaybackController(this)
    val reconnectController = ReconnectController(this)

    fun currentState(): CoyoteV3State = mutableState.value

    fun updateState(transform: (CoyoteV3State) -> CoyoteV3State) {
        mutableState.value = transform(mutableState.value)
    }

    suspend fun emit(event: CoyoteV3Event) {
        mutableEvents.emit(event)
    }

    fun tryEmit(event: CoyoteV3Event) {
        mutableEvents.tryEmit(event)
    }

    fun checkOpen() {
        if (closed) throw RuntimeException.Closed()
    }

    suspend fun initialize() {
        initialization.run()
    }

    suspend fun updateBattery(battery: BatteryLevel) {
        updateState {
            it.copy(
                common = it.common.copy(
                    battery = Observed.Known(BatteryState(battery)),
                    revision = it.common.revision + 1,
                ),
            )
        }
        tryEmit(CoyoteV3Event.Device(DeviceEvent.BatteryChanged(BatteryState(battery))))
    }

    fun close() {
        if (closed) return
        closed = true
        playJob?.cancel()
        notifyJob?.cancel()
        batteryJob?.cancel()
        scope.launch {
            runCatching {
                link.write(
                    GattCharacteristic(SERVICE_CONTROL, CHAR_WRITE),
                    B0Command(
                        Sequence.ZERO,
                        StereoIntensityOp(
                            IntensityOp.Set(Intensity.MIN),
                            IntensityOp.Set(Intensity.MIN)
                        ),
                        StereoWaveform.silent(),
                    ).encode(),
                    WriteMode.Auto,
                )
                link.disconnect()
            }
            emit(CoyoteV3Event.Closed)
            scope.cancel()
        }
    }

}
