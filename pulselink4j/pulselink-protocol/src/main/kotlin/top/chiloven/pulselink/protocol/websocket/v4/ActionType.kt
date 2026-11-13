package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException

/** Supported V4 device operation types. */
public enum class ActionType(public val wireValue: Int) {

    AppendPulseData(0),
    AddIntensity(3),
    SetTempIntensity(4),
    SetIntensity(7);

    public companion object {

        public fun fromWire(value: Int): ActionType =
            entries.firstOrNull { it.wireValue == value }
                ?: throw ProtocolException.UnsupportedActionType(value)

    }

}
