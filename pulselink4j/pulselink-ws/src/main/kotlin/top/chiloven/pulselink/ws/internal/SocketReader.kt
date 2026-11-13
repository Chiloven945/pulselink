package top.chiloven.pulselink.ws.internal

import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class SocketReader(
    private val session: DefaultClientWebSocketSession,
    private val scope: CoroutineScope,
    private val onText: suspend (String) -> Unit,
    private val onClose: suspend (Frame.Close) -> Unit,
    private val onError: suspend (Throwable) -> Unit,
) {

    fun start(): Job =
        scope.launch {
            try {
                for (frame in session.incoming) {
                    if (frame is Frame.Text) onText(frame.readText())
                    if (frame is Frame.Close) {
                        onClose(frame)
                        break
                    }
                }
            } catch (error: Throwable) {
                if (error !is CancellationException) {
                    onError(error)
                }
            }
        }

}
