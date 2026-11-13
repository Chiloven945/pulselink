package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.ScanFilter
import java.util.*

/** A local BLE device profile. */
public data class DeviceProfile(
    public val kind: DeviceKind,
    public val namePrefixes: List<String>,
    public val serviceUuids: List<UUID>,
) {

    /** Name-first scan filter. */
    public fun scanFilter(): ScanFilter =
        ScanFilter(namePrefixes = namePrefixes)

    /** Service-UUID fallback scan filter. */
    public fun fallbackScanFilter(): ScanFilter =
        ScanFilter(serviceUuids = serviceUuids)

    /** Returns whether [device] belongs to this profile. */
    public fun matches(device: DiscoveredDevice): Boolean {
        val name = device.name
        return (name != null && namePrefixes.any(name::startsWith)) ||
                (kind == DeviceKind.CoyoteV3 &&
                        name == null &&
                        serviceUuids.any(device.serviceUuids::contains)) ||
                classify(device) == kind
    }

    /** Initial capabilities for this kind. */
    public fun initialCapabilities(): DeviceCapabilities = when (kind) {
        DeviceKind.CoyoteV2 -> DeviceCapabilities(
            channels = ChannelSet.AB,
            battery = true,
            intensity = IntensityCapabilities(
                absolute = true,
                max = Intensity.MAX
            ),
        )

        DeviceKind.CoyoteV3 -> DeviceCapabilities(
            channels = ChannelSet.AB,
            battery = true,
            intensity = IntensityCapabilities(
                absolute = true,
                relative = true,
                temporary = true,
                max = Intensity.MAX
            ),
            waveform = WaveformCapabilities(v3Frames = true),
            settings = true,
        )

        DeviceKind.Opossum -> DeviceCapabilities(
            channels = ChannelSet.AB,
            battery = true,
            intensity = IntensityCapabilities(
                absolute = true,
                max = Intensity.MAX
            ),
            vibration = true,
            buttons = true,
        )

        DeviceKind.Civet -> DeviceCapabilities(
            channels = ChannelSet.NONE,
            battery = true,
            pressure = true,
        )

        DeviceKind.PawPrints -> DeviceCapabilities(
            channels = ChannelSet.NONE,
            battery = true,
            pressure = true,
            buttons = true,
        )

        is DeviceKind.Unknown -> DeviceCapabilities()
    }

}
