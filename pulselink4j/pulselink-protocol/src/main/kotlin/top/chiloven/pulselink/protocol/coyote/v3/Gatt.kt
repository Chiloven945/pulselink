package top.chiloven.pulselink.protocol.coyote.v3

import java.util.*

public const val SERVICE_CONTROL_ID: String = "0000180c-0000-1000-8000-00805f9b34fb"
public const val CHAR_WRITE_ID: String = "0000150a-0000-1000-8000-00805f9b34fb"
public const val CHAR_NOTIFY_ID: String = "0000150b-0000-1000-8000-00805f9b34fb"
public const val SERVICE_BATTERY_ID: String = "0000180a-0000-1000-8000-00805f9b34fb"
public const val CHAR_BATTERY_ID: String = "00001500-0000-1000-8000-00805f9b34fb"

public val SERVICE_CONTROL: UUID = UUID.fromString(SERVICE_CONTROL_ID)
public val CHAR_WRITE: UUID = UUID.fromString(CHAR_WRITE_ID)
public val CHAR_NOTIFY: UUID = UUID.fromString(CHAR_NOTIFY_ID)
public val SERVICE_BATTERY: UUID = UUID.fromString(SERVICE_BATTERY_ID)
public val CHAR_BATTERY: UUID = UUID.fromString(CHAR_BATTERY_ID)

public const val DEVICE_NAME: String = "47L121000"
public const val SENSOR_NAME: String = "47L120100"
