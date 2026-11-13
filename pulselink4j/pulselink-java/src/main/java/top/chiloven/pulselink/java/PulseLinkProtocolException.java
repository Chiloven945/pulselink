package top.chiloven.pulselink.java;

import java.io.Serial;

/** A protocol encoding, decoding, or validation failure. */
public final class PulseLinkProtocolException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkProtocolException(String message, Throwable cause) {
        super(message, cause);
    }

}
