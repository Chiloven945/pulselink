package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.CoreException
import top.chiloven.pulselink.device.DeviceError
import top.chiloven.pulselink.java.*
import top.chiloven.pulselink.pattern.PatternException
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.transport.TransportException
import top.chiloven.pulselink.ws.WsError
import top.chiloven.pulselink.core.PulseLinkException as KotlinPulseLinkException
import top.chiloven.pulselink.runtime.RuntimeException as KotlinRuntimeException

internal fun Throwable.toPulseLinkException(): PulseLinkException =
    when (this) {
        is CoreException -> PulseLinkCoreException(message ?: "core error", this)
        is PatternException -> PulseLinkPatternException(message ?: "pattern error", this)
        is ProtocolException -> PulseLinkProtocolException(message ?: "protocol error", this)
        is TransportException -> PulseLinkTransportException(message ?: "transport error", this)
        is DeviceError -> PulseLinkDeviceException(message ?: "device error", this)
        is KotlinRuntimeException -> PulseLinkRuntimeException(message ?: "runtime error", this)
        is WsError -> PulseLinkWebSocketException(message ?: "websocket error", this)
        is PulseLinkException -> this
        is KotlinPulseLinkException -> PulseLinkException(message ?: "pulselink error", this)
        else -> PulseLinkException(message ?: "unexpected error", this)
    }
