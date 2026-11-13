package top.chiloven.pulselink.ws

import top.chiloven.pulselink.protocol.websocket.v4.AppEvent
import top.chiloven.pulselink.protocol.websocket.v4.DeviceOperate
import top.chiloven.pulselink.ws.internal.AppOperations
import top.chiloven.pulselink.ws.internal.DeviceCache
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow

/** A connected DG-LAB app client. */
public class AppClient internal constructor(
    public val clientId: String,
    private val socket: Socket,
) {

    private val cache = DeviceCache()
    private val mutableEvents = MutableSharedFlow<WsEvent>(extraBufferCapacity = 64)
    private val operations = AppOperations(socket, clientId)

    /** Cached devices. */
    public fun devices(): List<RemoteDeviceSnapshot> =
        cache.snapshot()

    /** Requests a fresh device list. */
    public suspend fun requestDevices(): List<RemoteDeviceSnapshot> {
        cache.replace(operations.requestDevices())
        return cache.snapshot()
    }

    /** Returns a handle for a cached device. */
    public fun device(slotId: String): RemoteDeviceHandle? =
        cache.get(slotId)?.let { RemoteDeviceHandle(this, slotId) }

    /** Subscribes to app events. */
    public fun subscribe(): Flow<WsEvent> =
        mutableEvents.asSharedFlow()

    /** App-level ping. */
    public suspend fun ping(): Long =
        operations.ping()

    /** Clears operations. */
    public suspend fun clearOperate(target: ClearTarget) {
        operations.clear(target)
    }

    /** Submits a raw operation. */
    public suspend fun submitOperate(
        operate: DeviceOperate,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        operations.submit(operate, options)

    internal fun applyEvent(event: AppEvent) {
        cache.apply(event)?.let { mutableEvents.tryEmit(it) }
    }

    internal fun disconnected() {
        mutableEvents.tryEmit(WsEvent.Disconnected)
    }

}
