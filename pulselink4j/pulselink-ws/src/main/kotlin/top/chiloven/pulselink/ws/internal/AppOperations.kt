package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.protocol.websocket.v4.DeviceOperate
import top.chiloven.pulselink.protocol.websocket.v4.PulselinkJson
import top.chiloven.pulselink.protocol.websocket.v4.RemoteDevice
import top.chiloven.pulselink.ws.ClearTarget
import top.chiloven.pulselink.ws.OperationHandle
import top.chiloven.pulselink.ws.OperationOptions
import top.chiloven.pulselink.ws.Socket
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long

internal class AppOperations(
    private val socket: Socket,
    private val clientId: String,
) {

    suspend fun requestDevices(): List<RemoteDevice> {
        val result = socket.request(clientId, "devices.get")
        val array = result.jsonObject["devices"]?.jsonArray ?: return emptyList()
        return PulselinkJson.decodeFromJsonElement(
            ListSerializer(RemoteDevice.serializer()),
            array
        )
    }

    suspend fun ping(): Long =
        socket.request(clientId, "ping").jsonPrimitive.long

    suspend fun clear(target: ClearTarget) {
        socket.clear(clientId, target)
    }

    suspend fun submit(
        operate: DeviceOperate,
        options: OperationOptions,
    ): OperationHandle =
        socket.submit(clientId, operate, options)

}
