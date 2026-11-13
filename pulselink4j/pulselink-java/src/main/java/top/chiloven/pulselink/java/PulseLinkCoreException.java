package top.chiloven.pulselink.java;

import java.io.Serial;

/** A core-domain validation failure. */
public final class PulseLinkCoreException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkCoreException(String message, Throwable cause) {
        super(message, cause);
    }

}
