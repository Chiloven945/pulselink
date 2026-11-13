package top.chiloven.pulselink.java.ws;

/** Supported V4 device operation types. */
public enum ActionType {

    APPEND_PULSE_DATA(0),
    ADD_INTENSITY(3),
    SET_TEMP_INTENSITY(4),
    SET_INTENSITY(7);

    private final int wireValue;

    ActionType(int wireValue) {
        this.wireValue = wireValue;
    }

    public static ActionType fromWire(int value) {
        return switch (value) {
            case 0 -> APPEND_PULSE_DATA;
            case 3 -> ADD_INTENSITY;
            case 4 -> SET_TEMP_INTENSITY;
            case 7 -> SET_INTENSITY;
            default -> throw new IllegalArgumentException("unsupported action type " + value);
        };
    }

    public int wireValue() {
        return wireValue;
    }

}
