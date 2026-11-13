package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.ws.*
import java.util.*
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.ws.AppClient as KotlinAppClient

internal class AppClientAdapter(
    private val context: BridgeContext,
    private val app: KotlinAppClient,
) : AppClient {

    override fun clientId(): String = app.clientId

    override fun devices(): List<RemoteDevice> =
        app.devices().map {
            it.toJava()
        }

    override fun requestDevices(): CompletableFuture<List<RemoteDevice>> =
        context.future {
            app.requestDevices().map {
                it.toJava()
            }
        }

    override fun device(slotId: String): Optional<RemoteDeviceHandle> =
        app.device(slotId)
                ?.let {
                    Optional.of(RemoteDeviceHandleAdapter(context, it))
                }
            ?: Optional.empty()

    override fun events(): Flow.Publisher<WsEvent> =
        context.publisher(
            app.subscribe().map { it.toJava() },
            conflate = false
        )

    override fun ping(): CompletableFuture<Long> =
        context.future {
            app.ping()
        }

    override fun clearOperate(target: ClearTarget): CompletableFuture<Void> =
        context.futureVoid {
            app.clearOperate(target.toKotlin())
        }

    override fun submitOperate(
        operation: DeviceOperation,
        options: OperationOptions,
    ): CompletableFuture<OperationHandle> =
        context.future {
            OperationHandleAdapter(
                context,
                app.submitOperate(operation.toKotlin(), options.toKotlin()),
            )
        }

}
