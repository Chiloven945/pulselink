package top.chiloven.pulselink.ble.internal

import com.juul.kable.Advertisement
import com.juul.kable.Characteristic
import com.juul.kable.WriteType
import com.juul.kable.characteristicOf
import top.chiloven.pulselink.transport.*
import java.util.*
import kotlin.uuid.Uuid

internal fun UUID.toKableUuid(): Uuid =
    Uuid.parse(toString())

internal fun Uuid.toJavaUuid(): UUID =
    UUID.fromString(toString())

internal fun GattCharacteristic.toKable(): Characteristic =
    characteristicOf(service.toKableUuid(), characteristic.toKableUuid())

internal fun Advertisement.toDiscoveredDevice(): DiscoveredDevice =
    DiscoveredDevice(
        id = TransportDeviceId(identifier.toString()),
        name = name,
        rssi = rssi.takeUnless { it == Int.MIN_VALUE },
        serviceUuids = uuids.map(Uuid::toJavaUuid),
        manufacturerData = manufacturerData?.let {
            listOf(ManufacturerData(it.code, it.data.copyOf()))
        } ?: emptyList(),
        connectable = isConnectable,
    )

internal fun selectWriteType(
    info: CharacteristicInfo,
    mode: WriteMode
): WriteType =
    when (mode) {
        WriteMode.Auto -> when {
            info.writableWithoutResponse -> WriteType.WithoutResponse
            info.writable -> WriteType.WithResponse
            else -> throw TransportException.GattFailed("characteristic does not support writes")
        }

        WriteMode.WithResponse -> if (info.writable) {
            WriteType.WithResponse
        } else {
            throw TransportException.GattFailed("characteristic does not support write with response")
        }

        WriteMode.WithoutResponse -> if (info.writableWithoutResponse) {
            WriteType.WithoutResponse
        } else {
            throw TransportException.GattFailed("characteristic does not support write without response")
        }
    }
