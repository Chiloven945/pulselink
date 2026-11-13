package top.chiloven.pulselink.java.core;

/** A bitmask of physical buttons. */
public record ButtonMask(int bits) {

    public ButtonMask {
        if (bits < 0 || bits > 0xFFFF) {
            throw new IllegalArgumentException("button mask must be 0..65535");
        }
    }

    public boolean contains(ButtonMask other) {
        return (bits & other.bits) == other.bits;
    }

    public boolean isEmpty() {
        return bits == 0;
    }

}
