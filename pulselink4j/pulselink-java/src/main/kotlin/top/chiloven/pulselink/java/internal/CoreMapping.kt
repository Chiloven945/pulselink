package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.Observed
import top.chiloven.pulselink.java.core.*
import top.chiloven.pulselink.core.Channel as KotlinChannel
import top.chiloven.pulselink.core.ConnectionState as KotlinConnectionState
import top.chiloven.pulselink.core.DeviceEvent as KotlinDeviceEvent
import top.chiloven.pulselink.core.DeviceId as KotlinDeviceId
import top.chiloven.pulselink.core.DeviceInfo as KotlinDeviceInfo
import top.chiloven.pulselink.core.DeviceKind as KotlinDeviceKind
import top.chiloven.pulselink.core.DeviceState as KotlinDeviceState
import top.chiloven.pulselink.core.Intensity as KotlinIntensity
import top.chiloven.pulselink.core.IntensitySource as KotlinIntensitySource
import top.chiloven.pulselink.device.LocalEvent as KotlinLocalEvent

internal fun KotlinChannel.toJava(): Channel =
    when (this) {
        KotlinChannel.A -> Channel.A
        KotlinChannel.B -> Channel.B
    }

internal fun Channel.toKotlin(): KotlinChannel =
    when (this) {
        Channel.A -> KotlinChannel.A
        Channel.B -> KotlinChannel.B
    }

internal fun KotlinConnectionState.toJava(): ConnectionState =
    when (this) {
        KotlinConnectionState.Disconnected -> ConnectionState.DISCONNECTED
        KotlinConnectionState.Connecting -> ConnectionState.CONNECTING
        KotlinConnectionState.Initializing -> ConnectionState.INITIALIZING
        KotlinConnectionState.Ready -> ConnectionState.READY
        KotlinConnectionState.Degraded -> ConnectionState.DEGRADED
        KotlinConnectionState.Reconnecting -> ConnectionState.RECONNECTING
        KotlinConnectionState.Closing -> ConnectionState.CLOSING
    }

internal fun KotlinIntensitySource.toJava(): IntensitySource =
    when (this) {
        KotlinIntensitySource.Device -> IntensitySource.DEVICE
        KotlinIntensitySource.Sdk -> IntensitySource.SDK
        KotlinIntensitySource.PhysicalControl -> IntensitySource.PHYSICAL_CONTROL
        KotlinIntensitySource.Unknown -> IntensitySource.UNKNOWN
    }

internal fun KotlinDeviceKind.toJava(): DeviceKind =
    when (this) {
        KotlinDeviceKind.CoyoteV2 -> DeviceKind.CoyoteV2()
        KotlinDeviceKind.CoyoteV3 -> DeviceKind.CoyoteV3()
        KotlinDeviceKind.PawPrints -> DeviceKind.PawPrints()
        KotlinDeviceKind.Civet -> DeviceKind.Civet()
        KotlinDeviceKind.Opossum -> DeviceKind.Opossum()
        is KotlinDeviceKind.Unknown -> DeviceKind.Unknown(value)
    }

internal fun DeviceKind.toKotlin(): KotlinDeviceKind =
    when (this) {
        is DeviceKind.CoyoteV2 -> KotlinDeviceKind.CoyoteV2
        is DeviceKind.CoyoteV3 -> KotlinDeviceKind.CoyoteV3
        is DeviceKind.PawPrints -> KotlinDeviceKind.PawPrints
        is DeviceKind.Civet -> KotlinDeviceKind.Civet
        is DeviceKind.Opossum -> KotlinDeviceKind.Opossum
        is DeviceKind.Unknown -> KotlinDeviceKind.Unknown(value())
    }

internal fun KotlinDeviceId.toJava(): DeviceId =
    when (this) {
        is KotlinDeviceId.Ble -> DeviceId.Ble(value.value)
        is KotlinDeviceId.Remote -> DeviceId.Remote(value.clientId, value.slotId)
    }

internal fun KotlinDeviceInfo.toJava(): DeviceInfo =
    DeviceInfo(
        id.toJava(),
        kind.toJava(),
        name,
        manufacturer,
        model,
        firmware,
    )

internal fun KotlinDeviceState.toJava(): DeviceState =
    DeviceState(
        revision,
        connection.toJava(),
        battery.valueOrNull()?.let { BatteryState(it.level.value) },
        channels.a.toJava(),
        channels.b.toJava(),
    )

private fun top.chiloven.pulselink.core.ChannelState.toJava(): ChannelState =
    ChannelState(
        intensity.valueOrNull()?.value,
        softLimit.valueOrNull()?.value,
    )

private fun Observed<KotlinIntensity>.toJavaInt(): Int? =
    valueOrNull()?.value

internal fun KotlinLocalEvent.toJava(): LocalEvent =
    when (this) {
        is KotlinLocalEvent.ConnectionChanged ->
            LocalEvent.ConnectionChanged(state.toJava())

        is KotlinLocalEvent.BatteryChanged ->
            LocalEvent.BatteryChanged(BatteryState(battery.level.value))

        is KotlinLocalEvent.IntensityChanged ->
            LocalEvent.IntensityChanged(channel.toJava(), intensity.value)

        is KotlinLocalEvent.PressureChanged ->
            LocalEvent.PressureChanged(Pressure(pressure.centiKpa))

        is KotlinLocalEvent.ButtonChanged ->
            LocalEvent.ButtonChanged(ButtonMask(buttons.bits))

        is KotlinLocalEvent.Triggered ->
            LocalEvent.Triggered(Trigger(trigger.eventId, trigger.parameter))

        KotlinLocalEvent.SensorData ->
            LocalEvent.SensorData()

        is KotlinLocalEvent.Error ->
            LocalEvent.Failure(message)

        KotlinLocalEvent.Closed ->
            LocalEvent.Closed()
    }

internal fun KotlinDeviceEvent.toJava(): DeviceEvent =
    when (this) {
        is KotlinDeviceEvent.ConnectionChanged ->
            DeviceEvent.ConnectionChanged(state.toJava())

        is KotlinDeviceEvent.BatteryChanged ->
            DeviceEvent.BatteryChanged(BatteryState(battery.level.value))

        is KotlinDeviceEvent.IntensityChanged ->
            DeviceEvent.IntensityChanged(
                channel.toJava(),
                intensity.toJavaInt(),
                source.toJava()
            )

        KotlinDeviceEvent.SettingsChanged ->
            DeviceEvent.SettingsChanged()

        is KotlinDeviceEvent.PressureChanged ->
            DeviceEvent.PressureChanged(pressure.valueOrNull()?.let { Pressure(it.centiKpa) })

        is KotlinDeviceEvent.ButtonChanged ->
            DeviceEvent.ButtonChanged(ButtonMask(buttons.bits))

        is KotlinDeviceEvent.Triggered ->
            DeviceEvent.Triggered(Trigger(trigger.eventId, trigger.parameter))

        KotlinDeviceEvent.SensorData ->
            DeviceEvent.SensorData()

        is KotlinDeviceEvent.Error ->
            DeviceEvent.Failure(message)
    }

internal fun KotlinDeviceEvent.toLocalEvent(): LocalEvent =
    when (this) {
        is KotlinDeviceEvent.ConnectionChanged ->
            LocalEvent.ConnectionChanged(state.toJava())

        is KotlinDeviceEvent.BatteryChanged ->
            LocalEvent.BatteryChanged(BatteryState(battery.level.value))

        is KotlinDeviceEvent.IntensityChanged ->
            LocalEvent.IntensityChanged(channel.toJava(), intensity.toJavaInt())

        KotlinDeviceEvent.SettingsChanged ->
            LocalEvent.SensorData()

        is KotlinDeviceEvent.PressureChanged ->
            LocalEvent.PressureChanged(pressure.valueOrNull()?.let { Pressure(it.centiKpa) })

        is KotlinDeviceEvent.ButtonChanged ->
            LocalEvent.ButtonChanged(ButtonMask(buttons.bits))

        is KotlinDeviceEvent.Triggered ->
            LocalEvent.Triggered(Trigger(trigger.eventId, trigger.parameter))

        KotlinDeviceEvent.SensorData ->
            LocalEvent.SensorData()

        is KotlinDeviceEvent.Error ->
            LocalEvent.Failure(message)
    }
