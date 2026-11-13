package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.BatteryLevel
import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.device.DeviceError
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.device.LocalSession
import top.chiloven.pulselink.device.classify
import top.chiloven.pulselink.protocol.coyote.v2.PwmAb2
import top.chiloven.pulselink.protocol.pawprints.Command50
import top.chiloven.pulselink.runtime.CoyoteV3Config
import top.chiloven.pulselink.transport.Backend
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.WriteMode
import top.chiloven.pulselink.protocol.civet.D0Message as CivetD0
import top.chiloven.pulselink.protocol.opossum.Message as OpossumMessage
import top.chiloven.pulselink.protocol.pawprints.Message as PawMessage
import top.chiloven.pulselink.runtime.Client as RuntimeClient

internal class SessionFactory(
    private val backend: Backend,
    private val runtime: RuntimeClient,
) {

    suspend fun connect(device: DiscoveredDevice): LocalSession =
        when (val kind = classify(device)) {
            DeviceKind.CoyoteV3 -> connectCoyoteV3(device)
            DeviceKind.CoyoteV2 -> connectCoyoteV2(device)
            DeviceKind.Opossum -> connectOpossum(device)
            DeviceKind.Civet -> connectCivet(device)
            DeviceKind.PawPrints -> connectPawPrints(device)
            is DeviceKind.Unknown -> throw DeviceError.Unsupported(kind)
        }

    suspend fun connectCoyoteV3(
        device: DiscoveredDevice,
        config: CoyoteV3Config = CoyoteV3Config(),
    ): LocalSession.CoyoteV3 {
        val session = runtime.connectCoyoteV3(device, config)
        return LocalSession.CoyoteV3(CoyoteV3SessionImpl(session))
    }

    private suspend fun connectCoyoteV2(device: DiscoveredDevice): LocalSession.CoyoteV2 {
        val link = backend.connect(device)
        val base = BaseSession(
            link,
            GattIds.PWM_SERVICE,
            GattIds.PWM_AB2,
            decoder = { session, bytes ->
                runCatching { PwmAb2.decode(bytes) }
                        .onSuccess { value ->
                            session.updateIntensity(
                                Channel.A,
                                Intensity.of(value.a.display())
                            )
                            session.updateIntensity(
                                Channel.B,
                                Intensity.of(value.b.display())
                            )
                        }
            },
        )
        base.initialize()
        return LocalSession.CoyoteV2(CoyoteV2SessionImpl(base))
    }

    private suspend fun connectOpossum(device: DiscoveredDevice): LocalSession.Opossum {
        val link = backend.connect(device)
        val base = BaseSession(
            link,
            GattIds.CONTROL_SERVICE,
            GattIds.CONTROL_NOTIFY,
            decoder = { session, bytes ->
                runCatching { OpossumMessage.decode(bytes) }
                        .onSuccess { message ->
                            when (message) {
                                is OpossumMessage.B3Value -> {
                                    session.updateIntensity(
                                        Channel.A,
                                        Intensity.of(message.message.a)
                                    )
                                    session.updateIntensity(
                                        Channel.B,
                                        Intensity.of(message.message.b)
                                    )
                                }

                                is OpossumMessage.D0Value -> session.emit(
                                    LocalEvent.ButtonChanged(
                                        message.message.buttons
                                    )
                                )
                            }
                        }
            },
        )
        base.initialize()
        return LocalSession.Opossum(OpossumSessionImpl(base))
    }

    private suspend fun connectCivet(device: DiscoveredDevice): LocalSession.Civet {
        val link = backend.connect(device)
        val base = BaseSession(
            link,
            GattIds.CONTROL_SERVICE,
            GattIds.CONTROL_NOTIFY,
            decoder = { session, bytes ->
                runCatching { CivetD0.decode(bytes) }
                        .onSuccess { message ->
                            session.emit(LocalEvent.PressureChanged(message.pressure))
                        }
            },
        )
        base.initialize()
        return LocalSession.Civet(CivetSessionImpl(base))
    }

    private suspend fun connectPawPrints(device: DiscoveredDevice): LocalSession.PawPrints {
        val link = backend.connect(device)
        val base = BaseSession(
            link,
            GattIds.CONTROL_SERVICE,
            GattIds.CONTROL_NOTIFY,
            decoder = { session, bytes ->
                runCatching { PawMessage.decode(bytes) }
                        .onSuccess { message ->
                            when (message) {
                                is PawMessage.Info -> session.updateBattery(
                                    BatteryLevel.of(
                                        message.message.battery.coerceIn(0, 100)
                                    )
                                )

                                is PawMessage.Activated -> session.emit(
                                    LocalEvent.Triggered(
                                        top.chiloven.pulselink.core.Trigger(
                                            message.message.eventId,
                                            message.message.parameter
                                        )
                                    )
                                )

                                is PawMessage.Deactivated -> session.emit(
                                    LocalEvent.Triggered(
                                        top.chiloven.pulselink.core.Trigger(
                                            message.message.eventId,
                                            0
                                        )
                                    )
                                )

                                is PawMessage.Parameter -> session.emit(LocalEvent.SensorData)
                                is PawMessage.Sensor -> session.emit(LocalEvent.SensorData)
                                is PawMessage.Angles -> session.emit(LocalEvent.SensorData)
                            }
                        }
            },
            initCommand = {
                link.write(
                    GattCharacteristic(
                        GattIds.CONTROL_WRITE,
                        GattIds.CONTROL_WRITE,
                    ),
                    Command50.noTrigger(0).asBytes(),
                    WriteMode.Auto,
                )
            },
        )
        base.initialize()
        return LocalSession.PawPrints(PawPrintsSessionImpl(base))
    }

}
