package top.chiloven.pulselink.transport

/** Re-establishes a previously known link. */
public interface Connector {

    public suspend fun reconnect(target: ReconnectTarget): GattLink

}
