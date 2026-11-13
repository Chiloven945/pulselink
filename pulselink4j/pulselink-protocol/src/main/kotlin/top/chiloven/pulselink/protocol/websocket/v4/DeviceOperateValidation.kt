package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException

internal fun validateDeviceOperate(operate: DeviceOperate): DeviceOperate {
    if (operate.slotId.isEmpty()) {
        throw ProtocolException.InvalidParameters("slotId must not be empty")
    }

    if (operate.channel !in 0..1) {
        throw ProtocolException.ValueOutOfRange(
            "device channel",
            operate.channel.toLong(),
            0,
            1
        )
    }

    operate.priority?.let {
        if (it !in 0..2) throw ProtocolException.ValueOutOfRange(
            "device operation priority",
            it.toLong(),
            0,
            2
        )
    }

    when (operate.action) {
        ActionType.AppendPulseData -> validatePulseFrames(operate)
        ActionType.AddIntensity -> {
            val v = (operate.value as? OperateValue.Intensity)?.value
                ?: throw ProtocolException.InvalidParameters("AddIntensity requires an integer value")
            if (v !in -200..200) throw ProtocolException.ValueOutOfRange(
                "AddIntensity value",
                v.toLong(),
                -200,
                200
            )
        }

        ActionType.SetTempIntensity -> {
            val v = (operate.value as? OperateValue.Intensity)?.value
                ?: throw ProtocolException.InvalidParameters("SetTempIntensity requires an integer value")
            if (v !in 0..200) throw ProtocolException.ValueOutOfRange(
                "SetTempIntensity value",
                v.toLong(),
                0,
                200
            )
        }

        ActionType.SetIntensity -> {
            val v = (operate.value as? OperateValue.Intensity)?.value
                ?: throw ProtocolException.InvalidParameters("SetIntensity requires an integer value")
            if (v != 0) throw ProtocolException.InvalidParameters("SetIntensity accepts only value 0")
        }
    }
    return operate
}

private fun validatePulseFrames(operate: DeviceOperate) {
    val version = operate.ver ?: 3
    if (version !in 2..3)
        throw ProtocolException.ValueOutOfRange(
            "pulse frame version",
            version.toLong(),
            2,
            3
        )
    val expected = if (version == 2) 3 else 8
    val frames = (operate.value as? OperateValue.PulseFrames)?.frames
        ?: throw ProtocolException.InvalidParameters("AppendPulseData requires pulse frames")
    if (frames.isEmpty())
        throw ProtocolException.InvalidPulseFrame("AppendPulseData requires at least one frame")
    frames.forEach { it.validate(expected) }
}
