package top.chiloven.pulselink.ws

import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import top.chiloven.pulselink.protocol.websocket.v4.DeviceOperate
import top.chiloven.pulselink.ws.internal.*
import kotlin.time.Duration
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import kotlinx.serialization.json.JsonElement

/** A DG-LAB V4 controller socket. */
public class Socket private constructor(
    private val client: HttpClient,
    private val session: DefaultClientWebSocketSession,
    private val options: SocketOptions,
) : AutoCloseable {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val ids = RequestIds()
    private val writer = SocketWriter(session) { closed }
    private val rpc = RpcManager(ids)
    private val operations = OperationManager(ids, rpc)
    private val registry = AppRegistry(this)
    private val hello = CompletableDeferred<Unit>()
    private val mutableState = MutableStateFlow(SocketState.Connecting)
    private val mutableEvents = MutableSharedFlow<SocketEvent>(extraBufferCapacity = 64)
    private var targetIdValue: String? = null
    private var readerJob: Job? = null
    private var closed = false

    private val dispatcher = FrameDispatcher(
        state = mutableState,
        events = mutableEvents,
        registry = registry,
        rpc = rpc,
        operations = operations,
        onHello = { clientId ->
            targetIdValue = clientId
            hello.complete(Unit)
        },
        onCleanup = ::cleanup,
    )

    private val reader = SocketReader(
        session = session,
        scope = scope,
        onText = dispatcher::dispatch,
        onClose = { frame ->
            val reason = frame.readReason()
            mutableEvents.emit(
                SocketEvent.Closed(
                    reason?.code?.toInt(),
                    reason?.message ?: ""
                )
            )
            cleanup()
        },
        onError = { error ->
            mutableEvents.emit(SocketEvent.Error(error.message ?: "reader failed"))
            cleanup()
        },
    )

    /** The controller target id assigned by the relay. */
    public val targetId: String get() = targetIdValue ?: ""

    /** Current socket state. */
    public val state: StateFlow<SocketState>
        get() = mutableState.asStateFlow()

    /** Lifecycle events. */
    public val events: SharedFlow<SocketEvent>
        get() = mutableEvents.asSharedFlow()

    /** Waits for the first app client. */
    public suspend fun waitForClient(): AppClient {
        registry.first()?.let { return it }
        return registry.awaitFirst()
    }

    /** Returns all connected app clients. */
    public fun clients(): List<AppClient> =
        registry.all()

    /** Looks up an app client by id. */
    public fun client(clientId: String): AppClient? =
        registry.get(clientId)

    /** Closes the socket and releases its client. */
    public suspend fun shutdown() {
        if (closed) return
        closed = true
        mutableState.value = SocketState.Closing
        readerJob?.cancel()
        client.close()
        mutableState.value = SocketState.Closed
        scope.cancel()
    }

    override fun close() {
        if (closed) return
        closed = true
        mutableState.value = SocketState.Closed
        readerJob?.cancel()
        client.close()
        scope.cancel()
    }

    internal suspend fun request(
        clientId: String,
        method: String,
        data: JsonElement? = null,
        timeout: Duration = options.responseTimeout,
    ): JsonElement =
        rpc.request(clientId, method, data, timeout, writer::send)

    internal suspend fun submit(
        clientId: String,
        operate: DeviceOperate,
        options: OperationOptions,
    ): OperationHandle =
        operations.submit(clientId, operate, options, writer::send)

    internal suspend fun clear(clientId: String, target: ClearTarget) {
        operations.clear(clientId, target, options.responseTimeout, writer::send)
    }

    private fun startReader() {
        readerJob = reader.start()
    }

    private fun cleanup() {
        mutableState.value = SocketState.Closed
        rpc.failAll()
        operations.failAll()
        registry.disconnectAll()
    }

    public companion object {

        /** Connects to a relay and waits for `hello`. */
        public suspend fun connect(url: String): Socket = connect(SocketOptions(url))

        /** Connects with explicit options. */
        public suspend fun connect(options: SocketOptions): Socket {
            val client = HttpClient(CIO) {
                install(WebSockets) {
                    options.pingInterval?.let { pingIntervalMillis = it.inWholeMilliseconds }
                    maxFrameSize = options.maxFrameSize
                }
            }
            val session = try {
                withTimeout(options.connectTimeout) { client.webSocketSession(options.url) }
            } catch (error: Throwable) {
                client.close()
                throw WsError.Transport(error.message ?: "connect failed")
            }
            val socket = Socket(client, session, options)
            socket.startReader()
            try {
                withTimeout(options.connectTimeout) { socket.hello.await() }
            } catch (_: Throwable) {
                socket.shutdown()
                throw WsError.NotPaired()
            }
            return socket
        }
    }

}
