package top.chiloven.pulselink.protocol.coyote.v2

import java.util.*

public const val SERVICE_PWM_ID: String = "0000180b-0000-1000-8000-00805f9b34fb"
public const val CHAR_PWM_AB2_ID: String = "00001504-0000-1000-8000-00805f9b34fb"
public const val CHAR_PWM_A34_ID: String = "00001505-0000-1000-8000-00805f9b34fb"
public const val CHAR_PWM_B34_ID: String = "00001506-0000-1000-8000-00805f9b34fb"
public const val SERVICE_BATTERY_ID: String = "0000180a-0000-1000-8000-00805f9b34fb"
public const val CHAR_BATTERY_ID: String = "00001500-0000-1000-8000-00805f9b34fb"

public val SERVICE_PWM: UUID = UUID.fromString(SERVICE_PWM_ID)
public val CHAR_PWM_AB2: UUID = UUID.fromString(CHAR_PWM_AB2_ID)
public val CHAR_PWM_A34: UUID = UUID.fromString(CHAR_PWM_A34_ID)
public val CHAR_PWM_B34: UUID = UUID.fromString(CHAR_PWM_B34_ID)
public val SERVICE_BATTERY: UUID = UUID.fromString(SERVICE_BATTERY_ID)
public val CHAR_BATTERY: UUID = UUID.fromString(CHAR_BATTERY_ID)

public const val DEVICE_NAME: String = "D-LAB ESTIM01"
