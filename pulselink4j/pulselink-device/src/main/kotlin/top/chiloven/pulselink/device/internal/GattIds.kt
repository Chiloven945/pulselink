package top.chiloven.pulselink.device.internal

import java.util.*

internal object GattIds {

    val CONTROL_SERVICE: UUID = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
    val CONTROL_WRITE: UUID = UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb")
    val CONTROL_NOTIFY: UUID = UUID.fromString("0000150b-0000-1000-8000-00805f9b34fb")

    val BATTERY_SERVICE: UUID = UUID.fromString("0000180a-0000-1000-8000-00805f9b34fb")
    val BATTERY_CHAR: UUID = UUID.fromString("00001500-0000-1000-8000-00805f9b34fb")

    val PWM_SERVICE: UUID = UUID.fromString("0000180b-0000-1000-8000-00805f9b34fb")
    val PWM_AB2: UUID = UUID.fromString("00001504-0000-1000-8000-00805f9b34fb")
    val PWM_A34: UUID = UUID.fromString("00001505-0000-1000-8000-00805f9b34fb")
    val PWM_B34: UUID = UUID.fromString("00001506-0000-1000-8000-00805f9b34fb")

}
