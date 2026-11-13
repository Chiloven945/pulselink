package top.chiloven.pulselink.java;

import java.io.Serial;

/** Thrown when an operation is attempted after a client, session, or socket was closed. */
public final class PulseLinkClosedException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkClosedException() {
        super("pulselink resource is closed");
    }

}
