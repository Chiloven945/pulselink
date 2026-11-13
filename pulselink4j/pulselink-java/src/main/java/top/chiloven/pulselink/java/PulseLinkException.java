package top.chiloven.pulselink.java;

import java.io.Serial;

/**
 * Base class for all unchecked exceptions thrown by the Java extension.
 */
public class PulseLinkException extends RuntimeException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkException(String message) {
        super(message);
    }

    public PulseLinkException(String message, Throwable cause) {
        super(message, cause);
    }

}
