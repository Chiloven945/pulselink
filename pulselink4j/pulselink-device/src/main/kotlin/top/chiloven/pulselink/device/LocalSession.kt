package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.device.internal.*
import top.chiloven.pulselink.pattern.Pattern
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.coyote.v2.PwmPulse
import top.chiloven.pulselink.protocol.pawprints.Command50
import top.chiloven.pulselink.runtime.CoyoteV3Session
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

/** A connected local device session. */
public sealed interface LocalSession : AutoCloseable {

    public val kind: DeviceKind
    public val state: StateFlow<DeviceState>
    public val events: SharedFlow<LocalEvent>
    public suspend fun shutdown()

    public class CoyoteV2 internal constructor(
        private val impl: CoyoteV2SessionImpl,
    ) : LocalSession {

        override val kind: DeviceKind get() = impl.kind
        override val state: StateFlow<DeviceState> get() = impl.state
        override val events: SharedFlow<LocalEvent> get() = impl.events

        public suspend fun setIntensity(channel: Channel, intensity: Intensity) {
            impl.setIntensity(channel, intensity)
        }

        public suspend fun setPulse(channel: Channel, pulse: PwmPulse) {
            impl.setPulse(channel, pulse)
        }

        public suspend fun setPulse(
            channel: Channel,
            interval: PulseInterval
        ) {
            impl.setPulse(channel, interval)
        }

        override suspend fun shutdown() {
            impl.shutdown()
        }

        override fun close() {
            impl.close()
        }

    }

    public class CoyoteV3 internal constructor(
        private val impl: CoyoteV3SessionImpl,
    ) : LocalSession {

        override val kind: DeviceKind get() = impl.kind
        override val state: StateFlow<DeviceState> get() = impl.state
        override val events: SharedFlow<LocalEvent> get() = impl.events

        public val runtime: CoyoteV3Session
            get() = impl.runtime

        public suspend fun setIntensity(
            channel: Channel,
            intensity: Intensity
        ) {
            impl.setIntensity(channel, intensity)
        }

        public suspend fun adjustIntensity(
            channel: Channel,
            delta: IntensityDelta
        ) {
            impl.adjustIntensity(channel, delta)
        }

        public suspend fun play(pattern: Pattern) {
            impl.play(pattern)
        }

        public suspend fun stop() {
            impl.stop()
        }

        public suspend fun reconnect() {
            impl.reconnect()
        }

        override suspend fun shutdown() {
            impl.shutdown()
        }

        override fun close() {
            impl.close()
        }

    }

    public class Opossum internal constructor(
        private val impl: OpossumSessionImpl,
    ) : LocalSession {

        override val kind: DeviceKind get() = impl.kind
        override val state: StateFlow<DeviceState> get() = impl.state
        override val events: SharedFlow<LocalEvent> get() = impl.events

        public suspend fun setIntensity(
            channel: Channel,
            intensity: Intensity
        ) {
            impl.setIntensity(channel, intensity)
        }

        public suspend fun pulse(
            channel: Channel,
            values: List<Int>
        ) {
            impl.pulse(channel, values)
        }

        public suspend fun display(
            a: Intensity,
            b: Intensity
        ) {
            impl.display(a, b)
        }

        override suspend fun shutdown() {
            impl.shutdown()
        }

        override fun close() {
            impl.close()
        }

    }

    public class Civet internal constructor(
        private val impl: CivetSessionImpl,
    ) : LocalSession {

        override val kind: DeviceKind get() = impl.kind
        override val state: StateFlow<DeviceState> get() = impl.state
        override val events: SharedFlow<LocalEvent> get() = impl.events

        public suspend fun startPressure(color: Int = 0) {
            impl.startPressure(color)
        }

        public suspend fun stopPressure(color: Int = 0) {
            impl.stopPressure(color)
        }

        public suspend fun resetPressure() {
            impl.resetPressure()
        }

        public suspend fun setOrientation(value: Int) {
            impl.setOrientation(value)
        }

        override suspend fun shutdown() {
            impl.shutdown()
        }

        override fun close() {
            impl.close()
        }

    }

    public class PawPrints internal constructor(
        private val impl: PawPrintsSessionImpl,
    ) : LocalSession {

        override val kind: DeviceKind get() = impl.kind
        override val state: StateFlow<DeviceState> get() = impl.state
        override val events: SharedFlow<LocalEvent> get() = impl.events

        public suspend fun noTrigger(color: Int = 0) {
            impl.noTrigger(color)
        }

        public suspend fun configure(command: Command50) {
            impl.configure(command)
        }

        public suspend fun resetParameter() {
            impl.resetParameter()
        }

        public suspend fun autoDetectAngles() {
            impl.autoDetectAngles()
        }

        public suspend fun shoulderLight(color: Int) {
            impl.shoulderLight(color)
        }

        public suspend fun shoulderLightFlash(colorA: Int, colorB: Int, speed: Int) {
            impl.shoulderLightFlash(colorA, colorB, speed)
        }

        override suspend fun shutdown() {
            impl.shutdown()
        }

        override fun close() {
            impl.close()
        }

    }

}
