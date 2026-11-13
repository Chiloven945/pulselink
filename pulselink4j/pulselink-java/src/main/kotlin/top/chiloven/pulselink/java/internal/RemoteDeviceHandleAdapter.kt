package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.java.core.Channel
import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.ws.*
import java.time.Duration
import java.util.*
import java.util.concurrent.CompletableFuture
import kotlin.time.toKotlinDuration
import top.chiloven.pulselink.ws.RemoteDeviceHandle as KotlinRemoteDeviceHandle

internal class RemoteDeviceHandleAdapter(
    private val context: BridgeContext,
    private val handle: KotlinRemoteDeviceHandle,
) : RemoteDeviceHandle {

    override fun slotId(): String =
        handle.slotId

    override fun snapshot(): Optional<RemoteDevice> =
        handle.snapshot()
                ?.let {
                    Optional.of(it.toJava())
                }
            ?: Optional.empty()

    override fun kind(): DeviceKind =
        handle.kind().toJava()

    override fun typed(): Optional<TypedRemoteDevice> =
        handle.typed()
                ?.let {
                    Optional.of(it.toJava())
                }
            ?: Optional.empty()

    override fun resetIntensity(
        channel: Channel,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.resetIntensity(
                    channel.toKotlin(),
                    options.toKotlin()
                ),
            )
        }

    override fun addIntensity(
        channel: Channel,
        value: Int,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.addIntensity(
                    channel.toKotlin(),
                    value,
                    options.toKotlin()
                ),
            )
        }

    override fun reduceIntensity(
        channel: Channel,
        value: Int,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.reduceIntensity(
                    channel.toKotlin(),
                    value,
                    options.toKotlin()
                ),
            )
        }

    override fun setTempIntensity(
        channel: Channel,
        intensity: Int,
        duration: Duration,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.setTempIntensity(
                    channel.toKotlin(),
                    Intensity.of(intensity),
                    duration.toKotlinDuration(),
                    options.toKotlin(),
                ),
            )
        }

    override fun sendPulse(
        channel: Channel,
        duration: Duration,
        frames: MutableList<ByteArray>,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.sendPulse(
                    channel.toKotlin(),
                    duration.toKotlinDuration(),
                    frames,
                    options.toKotlin(),
                ),
            )
        }

    override fun sendPulseV2(
        channel: Channel,
        duration: Duration,
        frames: MutableList<ByteArray>,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                handle.sendPulseV2(
                    channel.toKotlin(),
                    duration.toKotlinDuration(),
                    frames,
                    options.toKotlin(),
                ),
            )
        }

}
