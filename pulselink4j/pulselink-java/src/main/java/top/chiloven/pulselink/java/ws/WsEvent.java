package top.chiloven.pulselink.java.ws;

import java.util.List;
import java.util.Map;
import org.jspecify.annotations.Nullable;

/** An app-scoped event. */
public sealed interface WsEvent {

    record DevicesSnapshot(List<RemoteDevice> devices) implements WsEvent {

        public DevicesSnapshot {
            devices = List.copyOf(devices);
        }

    }

    record DevicesPatch(
            List<RemoteDevice> added,
            List<String> removed
    ) implements WsEvent {

        public DevicesPatch {
            added = List.copyOf(added);
            removed = List.copyOf(removed);
        }

    }

    record SlotsPatch(List<Map<String, @Nullable Object>> slots) implements WsEvent {

        public SlotsPatch {
            slots = List.copyOf(slots);
        }

    }

    record CustomAction(int action) implements WsEvent {

    }

    record Disconnected() implements WsEvent {

    }

    record Failure(String message) implements WsEvent {

    }

}
