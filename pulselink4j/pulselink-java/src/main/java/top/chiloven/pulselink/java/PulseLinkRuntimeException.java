package top.chiloven.pulselink.java;

import java.io.Serial;

/** A Coyote V3 runtime failure. */
public final class PulseLinkRuntimeException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkRuntimeException(String message, Throwable cause) {
        super(message, cause);
    }

}
