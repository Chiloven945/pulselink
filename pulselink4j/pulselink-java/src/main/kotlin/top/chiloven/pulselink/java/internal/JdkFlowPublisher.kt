package top.chiloven.pulselink.java.internal

import java.util.concurrent.Executor
import java.util.concurrent.Flow
import java.util.concurrent.SubmissionPublisher
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.Flow as CoroutineFlow

internal class JdkFlowPublisher<T>(
    private val flow: CoroutineFlow<T>,
    private val scope: CoroutineScope,
    executor: Executor,
    private val conflate: Boolean,
) : Flow.Publisher<T>, AutoCloseable {

    private val publisher = SubmissionPublisher<T>(executor, 256)
    private var job: Job? = null

    @Synchronized
    override fun subscribe(subscriber: Flow.Subscriber<in T>) {
        publisher.subscribe(subscriber)
        if (job == null) {
            job = scope.launch {
                try {
                    val source = if (conflate) flow.conflate() else flow
                    source.collect { publisher.submit(it) }
                    publisher.close()
                } catch (_: CancellationException) {
                    publisher.close()
                } catch (error: Throwable) {
                    runCatching {
                        publisher.closeExceptionally(error.toPulseLinkException())
                    }
                }
            }
        }
    }

    @Synchronized
    override fun close() {
        job?.cancel()
        job = null
        runCatching { publisher.close() }
    }

}
