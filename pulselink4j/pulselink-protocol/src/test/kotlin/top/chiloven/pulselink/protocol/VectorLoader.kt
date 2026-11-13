package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.websocket.v4.PulselinkJson
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject

internal class VectorLoader {

    fun cases(path: String): List<JsonObject> {
        val text = checkNotNull(javaClass.getResource("/$path")) {
            "missing resource $path"
        }.readText()
        return PulselinkJson.parseToJsonElement(text)
                .jsonObject.getValue("cases")
                .jsonArray.map { it.jsonObject }
    }

    fun hex(bytes: ByteArray): String =
        bytes.joinToString("") {
            (it.toInt() and 0xFF)
                    .toString(16)
                    .padStart(2, '0')
                    .uppercase()
        }

    fun unhex(value: String): ByteArray =
        ByteArray(value.length / 2) {
            value.substring(it * 2, it * 2 + 2)
                    .toInt(16)
                    .toByte()
        }

}
