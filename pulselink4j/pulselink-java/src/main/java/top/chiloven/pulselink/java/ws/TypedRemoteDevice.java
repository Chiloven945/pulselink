package top.chiloven.pulselink.java.ws;

/** Typed remote snapshot wrapper. */
public sealed interface TypedRemoteDevice {

    RemoteDevice device();

    record Coyote(RemoteDevice device) implements TypedRemoteDevice {

    }

    record Civet(RemoteDevice device) implements TypedRemoteDevice {

    }

    record Opossum(RemoteDevice device) implements TypedRemoteDevice {

    }

    record Unknown(RemoteDevice device) implements TypedRemoteDevice {

    }

}
