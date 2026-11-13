package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.core.Observed
import top.chiloven.pulselink.pattern.Pattern
import top.chiloven.pulselink.pattern.PatternCursor
import top.chiloven.pulselink.protocol.coyote.v3.IntensityOp
import top.chiloven.pulselink.protocol.coyote.v3.StereoIntensityOp
import top.chiloven.pulselink.protocol.coyote.v3.StereoWaveform
import top.chiloven.pulselink.runtime.*
import kotlin.time.Duration
import kotlinx.coroutines.*
import kotlinx.coroutines.sync.withLock

internal class PlaybackController(private val core: SessionCore) {

    suspend fun play(
        pattern: Pattern,
        options: PlayOptions = PlayOptions()
    ): PlayHandle = core.playbackLock.withLock {
        core.checkOpen()
        if (core.playJob != null && !options.replace) throw RuntimeException.Busy()
        core.playJob?.cancel()
        core.playHandle?.finish(PlayEndReason.Replaced)
        val id = PatternId(++core.nextPatternId)
        val handle = PlayHandle(id)
        core.playId = id
        core.playHandle = handle
        core.updateState {
            it.copy(
                pattern = PatternPlaybackState(
                    id,
                    Duration.ZERO,
                    0,
                    pattern.looped
                )
            )
        }
        core.tryEmit(CoyoteV3Event.PatternStarted(id))
        core.playJob = core.scope.launch {
            val cursor = PatternCursor(pattern)
            var elapsed = Duration.ZERO
            try {
                while (true) {
                    val frame = cursor.currentFrame()
                    core.b0Writer.send(
                        StereoIntensityOp(
                            IntensityOp.Keep,
                            IntensityOp.Keep
                        ),
                        StereoWaveform(
                            frame.a.toWave(),
                            frame.b.toWave()
                        ),
                        awaitAck = false,
                    )
                    core.updateState {
                        it.copy(
                            pattern = PatternPlaybackState(
                                id,
                                elapsed,
                                cursor.frameIndex,
                                pattern.looped
                            ),
                        )
                    }
                    if (!cursor.advance()) {
                        handle.finish(PlayEndReason.Completed)
                        core.emit(CoyoteV3Event.PatternFinished(id, PlayEndReason.Completed))
                        break
                    }
                    delay(core.config.tickInterval)
                    elapsed += core.config.tickInterval
                }
            } catch (error: Throwable) {
                if (error is CancellationException) throw error
                handle.finish(PlayEndReason.Failed)
                core.emit(CoyoteV3Event.PatternFinished(id, PlayEndReason.Failed))
            }
            core.playJob = null
            core.playId = null
        }
        handle
    }

    suspend fun stop(): StopOutcome {
        core.checkOpen()
        core.playJob?.cancelAndJoin()
        core.playJob = null
        core.playId = null
        core.playHandle?.finish(PlayEndReason.Stopped)
        val acknowledged = try {
            core.b0Writer.send(
                StereoIntensityOp(
                    IntensityOp.Set(Intensity.MIN),
                    IntensityOp.Set(Intensity.MIN)
                ),
                StereoWaveform.silent(),
            )
            true
        } catch (_: Throwable) {
            currentCoroutineContext().ensureActive()
            false
        }
        return StopOutcome(
            acknowledged,
            ChannelMap(
                Observed.Known(Intensity.MIN),
                Observed.Known(Intensity.MIN),
            ),
        )
    }

}
