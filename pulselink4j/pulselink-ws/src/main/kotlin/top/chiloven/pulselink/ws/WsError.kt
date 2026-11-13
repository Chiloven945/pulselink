package top.chiloven.pulselink.ws

import top.chiloven.pulselink.core.PulseLinkException

/** WebSocket errors. */
public sealed class WsError(
    message: String,
    cause: Throwable? = null
) : PulseLinkException(message, cause) {

    public data class Transport(
        public val reason: String
    ) : WsError("websocket error: $reason")

    public data class Json(
        public val reason: String
    ) : WsError("JSON error: $reason")

    public data class Rpc(
        public val code: String,
        public val method: String
    ) : WsError("RPC error: $code")

    public data class RpcTimeout(
        public val reqId: String
    ) : WsError("RPC timeout for $reqId")

    public data class ClientNotFound(
        public val clientId: String
    ) : WsError("client not found: $clientId")

    public data class DeviceNotFound(
        public val slotId: String
    ) : WsError("device not found: $slotId")

    public data class DuplicateRequestId(
        public val reqId: String
    ) : WsError("duplicate request id: $reqId")

    public data class InvalidParams(
        public val reason: String
    ) : WsError("invalid parameters: $reason")

    public class Disconnected : WsError("not connected")

    public class NotPaired : WsError("socket is not paired")

    public class Busy : WsError("too many pending requests")

    public class Cancelled : WsError("operation cancelled")

    public class Closed : WsError("socket closed")

}
