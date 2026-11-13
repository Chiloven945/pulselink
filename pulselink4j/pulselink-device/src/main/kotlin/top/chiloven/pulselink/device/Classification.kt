package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.transport.DiscoveredDevice
import java.util.*

private val CONTROL: UUID = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
private val PWM: UUID = UUID.fromString("0000180b-0000-1000-8000-00805f9b34fb")

/** Classifies a discovered device by name first, then service UUID. */
public fun classify(device: DiscoveredDevice): DeviceKind =
    device.name.let { name ->
        when {
            name != null -> PROFILES
                    .firstOrNull { it.namePrefixes.any(name::startsWith) }
                    ?.kind
                ?: DeviceKind.Unknown(name)

            CONTROL in device.serviceUuids -> DeviceKind.CoyoteV3
            PWM in device.serviceUuids -> DeviceKind.CoyoteV2
            else -> DeviceKind.Unknown("unknown")
        }
    }

/** Returns the profile for a kind. */
public fun profileFor(kind: DeviceKind): DeviceProfile? =
    PROFILES.firstOrNull { it.kind == kind }
