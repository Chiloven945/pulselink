package top.chiloven.pulselink.ws.internal

import java.util.concurrent.atomic.AtomicLong

internal class RequestIds {

    private val counter = AtomicLong()

    fun next(): String =
        "req-${counter.incrementAndGet()}"

}
