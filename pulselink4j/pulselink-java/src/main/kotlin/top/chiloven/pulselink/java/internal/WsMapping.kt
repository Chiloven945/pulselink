package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.ws.*
import top.chiloven.pulselink.protocol.websocket.v4.DeviceOperate
import top.chiloven.pulselink.protocol.websocket.v4.PulseFrameWire
import top.chiloven.pulselink.ws.RemoteDeviceSnapshot
import top.chiloven.pulselink.ws.TypedRemoteSnapshot
import kotlin.time.toKotlinDuration
import kotlinx.serialization.json.*
import top.chiloven.pulselink.protocol.websocket.v4.ActionType as KotlinActionType
import top.chiloven.pulselink.protocol.websocket.v4.OperateValue as KotlinOperateValue
import top.chiloven.pulselink.protocol.websocket.v4.OperationEndReason as KotlinOperationEndReason
import top.chiloven.pulselink.ws.ClearTarget as KotlinClearTarget
import top.chiloven.pulselink.ws.OperationOptions as KotlinOperationOptions
import top.chiloven.pulselink.ws.OperationResult as KotlinOperationResult
import top.chiloven.pulselink.ws.SocketEvent as KotlinSocketEvent
import top.chiloven.pulselink.ws.SocketOptions as KotlinSocketOptions
import top.chiloven.pulselink.ws.SocketState as KotlinSocketState
import top.chiloven.pulselink.ws.WsEvent as KotlinWsEvent

internal fun SocketOptions.toKotlin(): KotlinSocketOptions =
    KotlinSocketOptions(
        url = url(),
        connectTimeout = connectTimeout().toKotlinDuration(),
        responseTimeout = responseTimeout().toKotlinDuration(),
        pingInterval = pingInterval()?.toKotlinDuration(),
        maxFrameSize = maxFrameSize(),
    )

internal fun KotlinSocketState.toJava(): SocketState =
    when (this) {
        KotlinSocketState.Connecting -> SocketState.CONNECTING
        KotlinSocketState.WaitingForPeer -> SocketState.WAITING_FOR_PEER
        KotlinSocketState.Ready -> SocketState.READY
        KotlinSocketState.Closing -> SocketState.CLOSING
        KotlinSocketState.Closed -> SocketState.CLOSED
    }

internal fun KotlinSocketEvent.toJava(): SocketEvent =
    when (this) {
        is KotlinSocketEvent.StateChanged -> SocketEvent.StateChanged(state.toJava())
        is KotlinSocketEvent.ClientAttached -> SocketEvent.ClientAttached(clientId)
        is KotlinSocketEvent.ClientDisconnected -> SocketEvent.ClientDisconnected(clientId)
        KotlinSocketEvent.Heartbeat -> SocketEvent.Heartbeat()
        is KotlinSocketEvent.Pong -> SocketEvent.Pong(ts)
        KotlinSocketEvent.IdleTimeout -> SocketEvent.IdleTimeout()
        is KotlinSocketEvent.Closed -> SocketEvent.Closed(code, reason)
        is KotlinSocketEvent.Error -> SocketEvent.Failure(message)
    }

internal fun RemoteDeviceSnapshot.toJava(): RemoteDevice =
    RemoteDevice(
        id,
        slotId,
        name,
        deviceType,
        props.toJavaMap(),
        slotState.toJavaMap(),
    )

internal fun TypedRemoteSnapshot.toJava(): TypedRemoteDevice =
    when (this) {
        is TypedRemoteSnapshot.Coyote -> TypedRemoteDevice.Coyote(snapshot.toJava())
        is TypedRemoteSnapshot.Civet -> TypedRemoteDevice.Civet(snapshot.toJava())
        is TypedRemoteSnapshot.Opossum -> TypedRemoteDevice.Opossum(snapshot.toJava())
        is TypedRemoteSnapshot.Unknown -> TypedRemoteDevice.Unknown(snapshot.toJava())
    }

internal fun KotlinWsEvent.toJava(): WsEvent =
    when (this) {
        is KotlinWsEvent.DevicesSnapshot ->
            WsEvent.DevicesSnapshot(devices.map { it.toJava() })

        is KotlinWsEvent.DevicesPatch ->
            WsEvent.DevicesPatch(added.map { it.toJava() }, removed)

        is KotlinWsEvent.SlotsPatch ->
            WsEvent.SlotsPatch(slots.map { it.toJavaMap() })

        is KotlinWsEvent.CustomAction ->
            WsEvent.CustomAction(action)

        KotlinWsEvent.Disconnected ->
            WsEvent.Disconnected()

        is KotlinWsEvent.Error ->
            WsEvent.Failure(message)
    }

internal fun ClearTarget.toKotlin(): KotlinClearTarget =
    when (this) {
        is ClearTarget.All ->
            KotlinClearTarget.All

        is ClearTarget.Device ->
            KotlinClearTarget.Device(slotId())

        is ClearTarget.Channel ->
            KotlinClearTarget.Channel(slotId(), channel().toKotlin())
    }

internal fun OperationOptions.toKotlin(): KotlinOperationOptions =
    KotlinOperationOptions(
        priority = priority(),
        immediate = immediate(),
    )

internal fun DeviceOperation.toKotlin(): DeviceOperate =
    DeviceOperate(
        slotId = slotId(),
        action = action().toKotlin(),
        channel = channel(),
        priority = priority(),
        durationMs = durationMs(),
        value = value().toKotlin(),
        immediate = immediate(),
        ver = ver(),
        seq = seq(),
    )

private fun OperateValue.toKotlin(): KotlinOperateValue =
    when (this) {
        is OperateValue.Intensity ->
            KotlinOperateValue.Intensity(value())

        is OperateValue.PulseFrames ->
            KotlinOperateValue.PulseFrames(
                frames().map { frame ->
                    PulseFrameWire.Numbers(
                        frame.map { it.toInt() and 0xFF }
                    )
                }
            )
    }

internal fun KotlinActionType.toJava(): ActionType =
    when (this) {
        KotlinActionType.AppendPulseData -> ActionType.APPEND_PULSE_DATA
        KotlinActionType.AddIntensity -> ActionType.ADD_INTENSITY
        KotlinActionType.SetTempIntensity -> ActionType.SET_TEMP_INTENSITY
        KotlinActionType.SetIntensity -> ActionType.SET_INTENSITY
    }

private fun ActionType.toKotlin(): KotlinActionType =
    when (this) {
        ActionType.APPEND_PULSE_DATA -> KotlinActionType.AppendPulseData
        ActionType.ADD_INTENSITY -> KotlinActionType.AddIntensity
        ActionType.SET_TEMP_INTENSITY -> KotlinActionType.SetTempIntensity
        ActionType.SET_INTENSITY -> KotlinActionType.SetIntensity
    }

internal fun KotlinOperationEndReason.toJava(): OperationEndReason =
    when (this) {
        KotlinOperationEndReason.Completed -> OperationEndReason.COMPLETED
        KotlinOperationEndReason.Cleared -> OperationEndReason.CLEARED
        KotlinOperationEndReason.Replaced -> OperationEndReason.REPLACED
        KotlinOperationEndReason.Cancelled -> OperationEndReason.CANCELLED
    }

internal fun KotlinOperationResult.toJava(): OperationResult =
    OperationResult(
        action.toJava(),
        reason.toJava(),
        slotId,
        channel?.toJava(),
    )

internal fun JsonObject.toJavaMap(): Map<String, Any?> =
    entries.associate { (key, value) ->
        key to value.toJavaValue()
    }

private fun JsonElement.toJavaValue(): Any? =
    when (this) {
        JsonNull -> null
        is JsonObject -> toJavaMap()
        is JsonArray -> map { it.toJavaValue() }
        is JsonPrimitive -> when {
            isString -> content
            booleanOrNull != null -> booleanOrNull!!
            longOrNull != null -> longOrNull!!
            doubleOrNull != null -> doubleOrNull!!
            else -> content
        }
    }
