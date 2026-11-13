package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.protocol.coyote.v3.*
import top.chiloven.pulselink.runtime.CoyoteV3Balance
import top.chiloven.pulselink.runtime.CoyoteV3Settings
import top.chiloven.pulselink.runtime.IntensityChange
import top.chiloven.pulselink.runtime.RuntimeException
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.WriteMode
import kotlinx.coroutines.sync.withLock

internal class IntensityController(private val core: SessionCore) {

    suspend fun setIntensity(channel: Channel, intensity: Intensity): IntensityChange {
        core.checkOpen()
        val limit = core.config.safety.maxIntensity[channel]
        if (intensity.value > limit.value) {
            throw RuntimeException.SafetyLimit("intensity ${intensity.value} exceeds ${limit.value}")
        }

        val ops = when (channel) {
            Channel.A -> StereoIntensityOp(
                IntensityOp.Set(intensity),
                IntensityOp.Keep
            )

            Channel.B -> StereoIntensityOp(
                IntensityOp.Keep,
                IntensityOp.Set(intensity)
            )
        }
        core.b0Writer.send(ops, StereoWaveform.silent())
        return IntensityChange(channel, intensity, intensity)
    }

    suspend fun adjustIntensity(channel: Channel, delta: IntensityDelta): IntensityChange {
        core.checkOpen()
        val maxDelta = core.config.safety.maxDelta?.get(channel)
        if (maxDelta != null && delta.magnitude() > maxDelta.magnitude()) {
            throw RuntimeException.SafetyLimit("delta ${delta.value} exceeds ${maxDelta.value}")
        }

        val current = core.currentState()
                .common.channel(channel)
                .intensity.valueOrNull()
            ?: Intensity.MIN
        val requested = current.value + delta.value
        val applied = Intensity.of(requested.coerceIn(0, 200))
        return setIntensity(channel, applied)
    }

    suspend fun setSoftLimit(channel: Channel, intensity: Intensity) {
        core.checkOpen()
        val settings = settingsOrThrow()
        val soft = when (channel) {
            Channel.A -> settings.softLimit.copy(a = intensity)
            Channel.B -> settings.softLimit.copy(b = intensity)
        }
        writeBf(
            soft,
            settings.frequencyBalance,
            settings.intensityBalance
        )
    }

    suspend fun setBalance(balance: CoyoteV3Balance) {
        core.checkOpen()
        val settings = settingsOrThrow()
        writeBf(
            settings.softLimit,
            balance.frequency,
            balance.intensity
        )
    }

    private suspend fun writeBf(
        soft: ChannelMap<Intensity>,
        frequency: ChannelMap<Int>,
        intensity: ChannelMap<Int>,
    ) {
        val command = BfCommand(
            soft,
            frequency,
            intensity
        )
        core.writeMutex.withLock {
            core.link.write(
                GattCharacteristic(
                    SERVICE_CONTROL,
                    CHAR_WRITE
                ),
                command.encode(),
                WriteMode.Auto
            )
        }
        core.updateState {
            it.copy(
                settings = Observed.Known(
                    CoyoteV3Settings(
                        soft,
                        frequency,
                        intensity
                    )
                ),
            )
        }
    }

    private fun settingsOrThrow(): CoyoteV3Settings =
        core.currentState().settings.valueOrNull() ?: throw RuntimeException.NotReady()

}
