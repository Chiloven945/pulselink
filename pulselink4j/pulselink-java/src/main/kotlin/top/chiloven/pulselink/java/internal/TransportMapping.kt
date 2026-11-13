package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.transport.DeviceProfile
import top.chiloven.pulselink.java.transport.DiscoveredDevice
import top.chiloven.pulselink.java.transport.ManufacturerData
import top.chiloven.pulselink.java.transport.ScanFilter
import top.chiloven.pulselink.transport.TransportDeviceId
import top.chiloven.pulselink.device.DeviceProfile as KotlinDeviceProfile
import top.chiloven.pulselink.transport.DiscoveredDevice as KotlinDiscoveredDevice
import top.chiloven.pulselink.transport.ManufacturerData as KotlinManufacturerData
import top.chiloven.pulselink.transport.ScanFilter as KotlinScanFilter

internal fun DiscoveredDevice.toKotlin(): KotlinDiscoveredDevice =
    KotlinDiscoveredDevice(
        id = TransportDeviceId(id()),
        name = name(),
        rssi = rssi(),
        serviceUuids = serviceUuids(),
        manufacturerData = manufacturerData().map { it.toKotlin() },
        connectable = connectable(),
    )

internal fun KotlinDiscoveredDevice.toJava(): DiscoveredDevice =
    DiscoveredDevice(
        id.value,
        name,
        rssi,
        serviceUuids,
        manufacturerData.map { it.toJava() },
        connectable,
    )

private fun ManufacturerData.toKotlin(): KotlinManufacturerData =
    KotlinManufacturerData(companyId(), data())

private fun KotlinManufacturerData.toJava(): ManufacturerData =
    ManufacturerData(companyId, data)

internal fun ScanFilter.toKotlin(): KotlinScanFilter =
    KotlinScanFilter(namePrefixes(), serviceUuids())

internal fun DeviceProfile.toKotlin(): KotlinDeviceProfile =
    KotlinDeviceProfile(
        kind().toKotlin(),
        namePrefixes(),
        serviceUuids()
    )
