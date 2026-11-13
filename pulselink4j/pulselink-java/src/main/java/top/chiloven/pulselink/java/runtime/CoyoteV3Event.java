package top.chiloven.pulselink.java.runtime;

import top.chiloven.pulselink.java.core.DeviceEvent;

import java.time.Duration;

/** Events emitted by a Coyote V3 session. */
public sealed interface CoyoteV3Event {

    record Device(DeviceEvent event) implements CoyoteV3Event {

    }

    record Reconnecting(
            int attempt,
            Duration delay
    ) implements CoyoteV3Event {

    }

    record Reconnected() implements CoyoteV3Event {

    }

    record ReconnectFailed(
            int attempts,
            Throwable error
    ) implements CoyoteV3Event {

    }

    record PatternStarted(long id) implements CoyoteV3Event {

    }

    record PatternFinished(
            long id,
            PlayEndReason reason
    ) implements CoyoteV3Event {

    }

    record SchedulerLag(Duration behind) implements CoyoteV3Event {

    }

    record AckTimeout(int sequence) implements CoyoteV3Event {

    }

    record B1Mismatch(
            int expected,
            int received
    ) implements CoyoteV3Event {

    }

    record Closed() implements CoyoteV3Event {

    }

}
