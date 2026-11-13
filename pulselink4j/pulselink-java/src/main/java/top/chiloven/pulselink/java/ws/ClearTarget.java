package top.chiloven.pulselink.java.ws;

/** The scope of a {@code device.op.clear}. */
public sealed interface ClearTarget {

    record All() implements ClearTarget {

    }

    record Device(String slotId) implements ClearTarget {

    }

    record Channel(
            String slotId,
            top.chiloven.pulselink.java.core.Channel channel
    ) implements ClearTarget {

    }

}
