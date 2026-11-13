package top.chiloven.pulselink.java;

import java.io.Serial;

/** A local device session failure. */
public final class PulseLinkDeviceException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkDeviceException(String message, Throwable cause) {
        super(message, cause);
    }

}
