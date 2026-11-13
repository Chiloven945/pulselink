package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.protocol.coyote.v3.*
import top.chiloven.pulselink.runtime.CoyoteV3Event
import top.chiloven.pulselink.runtime.RuntimeException
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.WriteMode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeout

internal class B0Writer(private val core: SessionCore) {

    private var sequence: Int = 0

    suspend fun send(
        intensity: StereoIntensityOp,
        waveform: StereoWaveform,
        awaitAck: Boolean = true,
    ) {
        core.checkOpen()
        val seq = nextSequence()
        val waiter = if (awaitAck) core.ackTracker.register(seq) else null
        try {
            core.writeMutex.withLock {
                core.link.write(
                    GattCharacteristic(SERVICE_CONTROL, CHAR_WRITE),
                    B0Command(seq, intensity, waveform).encode(),
                    WriteMode.Auto,
                )
            }
            if (waiter != null) {
                try {
                    withTimeout(core.config.ackTimeout) { waiter.await() }
                } catch (_: TimeoutCancellationException) {
                    core.ackTracker.remove(seq)
                    core.emit(CoyoteV3Event.AckTimeout(seq))
                    throw RuntimeException.AckTimeout(seq.value)
                } catch (error: CancellationException) {
                    core.ackTracker.remove(seq)
                    throw error
                }
            }
        } catch (error: Throwable) {
            if (waiter != null) core.ackTracker.remove(seq)
            throw error
        }
    }

    private suspend fun nextSequence(): Sequence {
        core.writeMutex.withLock {
            sequence = if (sequence >= Sequence.MAX) 1 else sequence + 1
        }
        return Sequence.of(sequence)
    }

}
