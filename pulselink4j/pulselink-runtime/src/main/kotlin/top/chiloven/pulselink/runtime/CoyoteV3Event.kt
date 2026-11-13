package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.DeviceEvent
import top.chiloven.pulselink.protocol.coyote.v3.Sequence
import kotlin.time.Duration

/** Events emitted by a Coyote V3 session. */
public sealed interface CoyoteV3Event {

    public data class Device(
        public val event: DeviceEvent
    ) : CoyoteV3Event

    public data class Reconnecting(
        public val attempt: Int,
        public val delay: Duration
    ) : CoyoteV3Event

    public data object Reconnected : CoyoteV3Event

    public data class ReconnectFailed(
        public val attempts: Int,
        public val error: Throwable
    ) : CoyoteV3Event

    public data class PatternStarted(
        public val id: PatternId
    ) : CoyoteV3Event

    public data class PatternFinished(
        public val id: PatternId,
        public val reason: PlayEndReason
    ) : CoyoteV3Event

    public data class SchedulerLag(
        public val behind: Duration
    ) : CoyoteV3Event

    public data class AckTimeout(
        public val sequence: Sequence
    ) : CoyoteV3Event

    public data class B1Mismatch(
        public val expected: Int,
        public val received: Int
    ) : CoyoteV3Event

    public data object Closed : CoyoteV3Event

}
