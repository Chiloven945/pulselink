package top.chiloven.pulselink.java;

import java.io.Serial;

/** A waveform/pattern validation failure. */
public final class PulseLinkPatternException extends PulseLinkException {

    @Serial private static final long serialVersionUID = 1L;

    public PulseLinkPatternException(String message, Throwable cause) {
        super(message, cause);
    }

}
