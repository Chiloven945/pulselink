package top.chiloven.pulselink.java.core;

/** A pressure reading in hundredths of a kilopascal. */
public record Pressure(int centiKpa) {

    /** Returns the pressure in kilopascals. */
    public float asKpa() {
        return centiKpa / 100.0f;
    }

}
