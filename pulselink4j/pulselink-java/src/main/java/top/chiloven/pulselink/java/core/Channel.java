package top.chiloven.pulselink.java.core;

/** A device output channel. */
public enum Channel {

    A(0, 0),
    B(1, 1);

    private final int index;
    private final int wireValue;

    Channel(int index, int wireValue) {
        this.index = index;
        this.wireValue = wireValue;
    }

    public static Channel fromWire(int value) {
        return switch (value) {
            case 0 -> A;
            case 1 -> B;
            default -> throw new IllegalArgumentException("invalid channel " + value);
        };
    }

    public int index() {
        return index;
    }

    public int wireValue() {
        return wireValue;
    }

}
