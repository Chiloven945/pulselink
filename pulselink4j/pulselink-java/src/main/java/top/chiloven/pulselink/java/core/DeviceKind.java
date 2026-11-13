package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** The kind of DG-LAB device. */
public sealed interface DeviceKind {

    static DeviceKind fromRemoteType(String value) {
        return switch (value) {
            case "COYOTE_020" -> new CoyoteV2();
            case "COYOTE_030" -> new CoyoteV3();
            case "BMTR_1" -> new Civet();
            case "OVC_1" -> new Opossum();
            default -> new Unknown(value);
        };
    }

    /** The DG-LAB V4 device type, when one exists. */
    default @Nullable String remoteType() {
        return switch (this) {
            case CoyoteV2 ignored -> "COYOTE_020";
            case CoyoteV3 ignored -> "COYOTE_030";
            case Civet ignored -> "BMTR_1";
            case Opossum ignored -> "OVC_1";
            case PawPrints ignored -> null;
            case Unknown ignored -> null;
        };
    }

    /** A human-readable name. */
    default String displayName() {
        return switch (this) {
            case CoyoteV2 ignored -> "Coyote V2";
            case CoyoteV3 ignored -> "Coyote V3";
            case PawPrints ignored -> "PawPrints";
            case Civet ignored -> "Civet";
            case Opossum ignored -> "Opossum";
            case Unknown unknown -> unknown.value();
        };
    }

    record CoyoteV2() implements DeviceKind {

    }

    record CoyoteV3() implements DeviceKind {

    }

    record PawPrints() implements DeviceKind {

    }

    record Civet() implements DeviceKind {

    }

    record Opossum() implements DeviceKind {

    }

    record Unknown(String value) implements DeviceKind {

    }

}
