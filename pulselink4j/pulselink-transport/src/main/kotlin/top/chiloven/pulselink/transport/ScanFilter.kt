package top.chiloven.pulselink.transport

import java.util.*

/** A filter applied during scanning and reconnection. */
public data class ScanFilter(
    public val namePrefixes: List<String> = emptyList(),
    public val serviceUuids: List<UUID> = emptyList(),
) {

    /**
     * Returns whether [device] matches this filter.
     *
     * An empty filter matches every device. Otherwise, a name-prefix match or
     * a service-UUID match is sufficient.
     */
    public fun matches(device: DiscoveredDevice): Boolean {
        if (namePrefixes.isEmpty() && serviceUuids.isEmpty()) return true
        val nameMatches = device.name?.let { namePrefixes.any(it::startsWith) } ?: false
        val serviceMatches = serviceUuids.any(device.serviceUuids::contains)
        return nameMatches || serviceMatches
    }

    public companion object {

        /** A filter that matches every device. */
        public fun all(): ScanFilter =
            ScanFilter()

    }

}
