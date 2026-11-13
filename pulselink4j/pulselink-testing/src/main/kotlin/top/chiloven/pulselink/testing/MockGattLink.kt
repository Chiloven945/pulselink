package top.chiloven.pulselink.testing

import top.chiloven.pulselink.transport.*
import kotlinx.coroutines.flow.*

/** A controllable in-memory [GattLink]. */
public class MockGattLink(
    override val deviceId: TransportDeviceId,
    private var characteristicMap: Map<GattCharacteristic, CharacteristicInfo> = emptyMap(),
) : GattLink {

    private val connected = MutableStateFlow(true)
    private val notifications = MutableSharedFlow<Pair<GattCharacteristic, ByteArray>>(
        extraBufferCapacity = 64,
    )
    private val writesList = mutableListOf<WriteRecord>()
    private val readData = mutableMapOf<GattCharacteristic, ByteArray>()
    private var failure: TransportException? = null
    private var mtuValue: Int? = 185

    /** Replaces the discovered characteristic map. */
    public fun setCharacteristics(characteristics: Map<GattCharacteristic, CharacteristicInfo>) {
        characteristicMap = characteristics
    }

    /** Injects a notification for [characteristic]. */
    public fun injectNotification(characteristic: GattCharacteristic, data: ByteArray) {
        notifications.tryEmit(characteristic to data)
    }

    /** Returns a snapshot of recorded writes. */
    public fun writes(): List<WriteRecord> =
        synchronized(writesList) { writesList.toList() }

    /** Makes the next write fail. */
    public fun failNextWrite(error: TransportException) {
        failure = error
    }

    /** Sets data returned by [read]. */
    public fun setReadData(characteristic: GattCharacteristic, data: ByteArray) {
        readData[characteristic] = data
    }

    /** Sets the reported MTU. */
    public fun setMtu(value: Int?) {
        mtuValue = value
    }

    /** Marks the link connected or disconnected. */
    public fun setConnected(value: Boolean) {
        connected.value = value
    }

    override val isConnected: Boolean get() = connected.value

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
        synchronized(writesList) {
            writesList += WriteRecord(characteristic, data.copyOf(), mode)
        }
        failure?.let {
            failure = null
            throw it
        }
        if (!connected.value) {
            throw TransportException.Disconnected()
        }
    }

    override suspend fun read(characteristic: GattCharacteristic): ByteArray {
        if (!connected.value) {
            throw TransportException.Disconnected()
        }
        return readData[characteristic]?.copyOf()
            ?: throw TransportException.GattFailed("no mock read data for $characteristic")
    }

    override fun subscribe(characteristic: GattCharacteristic): Flow<ByteArray> =
        notifications
                .filter { it.first == characteristic }
                .map { it.second.copyOf() }
                .takeWhile { connected.value }

    override suspend fun disconnect() {
        connected.value = false
    }

}
