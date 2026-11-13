package top.chiloven.pulselink.java;

import java.io.Serial;

/** A DG-LAB V4 WebSocket failure. */
public final class PulseLinkWebSocketException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkWebSocketException(String message, Throwable cause) {
        super(message, cause);
    }

}
