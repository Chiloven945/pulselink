package top.chiloven.pulselink.transport

/** The write mode used for a GATT characteristic. */
public enum class WriteMode {

    /** Choose based on characteristic properties. */
    Auto,

    /** Require a write response. */
    WithResponse,

    /** Use a write command without a response. */
    WithoutResponse,

}
