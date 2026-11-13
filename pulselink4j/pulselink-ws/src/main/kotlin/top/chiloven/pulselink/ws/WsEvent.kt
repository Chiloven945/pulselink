package top.chiloven.pulselink.ws

import kotlinx.serialization.json.JsonObject

/** An app-scoped event. */
public sealed interface WsEvent {

    public data class DevicesSnapshot(
        public val devices: List<RemoteDeviceSnapshot>
    ) : WsEvent

    public data class DevicesPatch(
        public val added: List<RemoteDeviceSnapshot>,
        public val removed: List<String>
    ) : WsEvent

    public data class SlotsPatch(
        public val slots: List<JsonObject>
    ) : WsEvent

    public data class CustomAction(
        public val action: Int
    ) : WsEvent

    public data object Disconnected : WsEvent

    public data class Error(
        public val message: String
    ) : WsEvent

}
