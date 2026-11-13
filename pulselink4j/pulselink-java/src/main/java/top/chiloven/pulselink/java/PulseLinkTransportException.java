package top.chiloven.pulselink.java;

import java.io.Serial;

/** A local transport failure. */
public final class PulseLinkTransportException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkTransportException(String message, Throwable cause) {
        super(message, cause);
    }

}
