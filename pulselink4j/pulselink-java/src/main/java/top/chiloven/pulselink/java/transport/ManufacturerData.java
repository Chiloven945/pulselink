package top.chiloven.pulselink.java.transport;

import java.util.Arrays;

/** Manufacturer-specific advertisement data. */
@SuppressWarnings("ArrayRecordComponent")
public record ManufacturerData(
        int companyId,
        byte[] data
) {

    public ManufacturerData {
        data = data.clone();
    }

    @Override
    public byte[] data() {
        return data.clone();
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof ManufacturerData that
                && companyId == that.companyId
                && Arrays.equals(data, that.data);
    }

    @Override
    public int hashCode() {
        return 31 * companyId + Arrays.hashCode(data);
    }

    @Override
    public String toString() {
        return "ManufacturerData(companyId=%d, bytes=%d)".formatted(companyId, data.length);
    }

}
