package top.chiloven.pulselink.protocol.pawprints

public data class D0Message(
    public val color: Int,
    public val sequence: Int,
    public val button: Int,
    public val acceleration: Int,
    public val x: Int,
    public val y: Int,
    public val z: Int,
    public val voltage: Int,
)
