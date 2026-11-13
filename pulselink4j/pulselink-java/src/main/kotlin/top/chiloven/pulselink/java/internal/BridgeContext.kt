package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.PulseLinkClosedException
import java.util.concurrent.*
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.*
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow as CoroutineFlow

internal class BridgeContext(
    private val scope: CoroutineScope,
    private val publisherExecutor: ExecutorService,
) : AutoCloseable {

    constructor() : this(
        CoroutineScope(SupervisorJob() + Dispatchers.IO),
        Executors.newVirtualThreadPerTaskExecutor(),
    )

    private val pending = ConcurrentHashMap.newKeySet<CompletableFuture<*>>()
    private val publishers = ConcurrentHashMap.newKeySet<JdkFlowPublisher<*>>()
    private val closed = AtomicBoolean(false)

    fun <T> future(block: suspend () -> T): CompletableFuture<T> {
        if (closed.get()) {
            return CompletableFuture<T>().also {
                it.completeExceptionally(PulseLinkClosedException())
            }
        }

        val future = CompletableFuture<T>()
        pending += future
        val job = scope.launch {
            try {
                future.complete(block())
            } catch (_: CancellationException) {
                future.cancel(false)
            } catch (error: Throwable) {
                future.completeExceptionally(error.toPulseLinkException())
            } finally {
                pending -= future
            }
        }
        future.whenComplete { _, _ ->
            pending -= future
            if (future.isCancelled) {
                job.cancel()
            }
        }
        return future
    }

    fun futureVoid(block: suspend () -> Unit): CompletableFuture<Void> {
        if (closed.get()) {
            return CompletableFuture<Void>().also {
                it.completeExceptionally(PulseLinkClosedException())
            }
        }

        val future = CompletableFuture<Void>()
        pending += future
        val job = scope.launch {
            try {
                block()
                future.complete(null)
            } catch (_: CancellationException) {
                future.cancel(false)
            } catch (error: Throwable) {
                future.completeExceptionally(error.toPulseLinkException())
            } finally {
                pending -= future
            }
        }
        future.whenComplete { _, _ ->
            pending -= future
            if (future.isCancelled) {
                job.cancel()
            }
        }
        return future
    }

    fun <T> publisher(flow: CoroutineFlow<T>, conflate: Boolean): Flow.Publisher<T> {
        checkOpen()
        val publisher = JdkFlowPublisher(flow, scope, publisherExecutor, conflate)
        publishers += publisher
        return publisher
    }

    fun checkOpen() {
        if (closed.get()) {
            throw PulseLinkClosedException()
        }
    }

    override fun close() {
        if (!closed.compareAndSet(false, true)) {
            return
        }
        pending.forEach { it.completeExceptionally(PulseLinkClosedException()) }
        pending.clear()
        publishers.forEach { it.close() }
        publishers.clear()
        scope.cancel()
        publisherExecutor.shutdownNow()
    }

}
