package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.DeviceKind
import java.util.*

private val CONTROL: UUID = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
private val PWM: UUID = UUID.fromString("0000180b-0000-1000-8000-00805f9b34fb")

public val COYOTE_V2: DeviceProfile = DeviceProfile(
    DeviceKind.CoyoteV2,
    listOf("D-LAB ESTIM01"),
    listOf(PWM)
)
public val COYOTE_V3: DeviceProfile = DeviceProfile(
    DeviceKind.CoyoteV3,
    listOf("47L121000"),
    listOf(CONTROL)
)
public val OPOSSUM: DeviceProfile = DeviceProfile(
    DeviceKind.Opossum,
    listOf("47L127000"),
    listOf(CONTROL)
)
public val CIVET: DeviceProfile = DeviceProfile(
    DeviceKind.Civet,
    listOf("47L124000"),
    listOf(CONTROL)
)
public val PAW_PRINTS: DeviceProfile = DeviceProfile(
    DeviceKind.PawPrints,
    listOf("47L120300", "47L120100"),
    listOf(CONTROL),
)

public val PROFILES: List<DeviceProfile> = listOf(
    COYOTE_V2,
    COYOTE_V3,
    OPOSSUM,
    CIVET,
    PAW_PRINTS
)
