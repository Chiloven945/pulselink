package top.chiloven.pulselink.ws.internal

import top.chiloven.pulselink.protocol.websocket.v4.AppEvent
import top.chiloven.pulselink.protocol.websocket.v4.RemoteDevice
import top.chiloven.pulselink.protocol.websocket.v4.SlotPatchEntry
import top.chiloven.pulselink.ws.RemoteDeviceSnapshot
import top.chiloven.pulselink.ws.WsEvent

internal class DeviceCache {

    private val devices = LinkedHashMap<String, RemoteDeviceSnapshot>()

    fun snapshot(): List<RemoteDeviceSnapshot> =
        devices.values.toList()

    fun get(slotId: String): RemoteDeviceSnapshot? =
        devices[slotId]

    fun replace(items: List<RemoteDevice>) {
        devices.clear()
        items.forEach { devices[it.slotId] = it.toSnapshot() }
    }

    fun apply(event: AppEvent): WsEvent? =
        when (event) {
            is AppEvent.DevicesSnapshot -> {
                replace(event.devices)
                WsEvent.DevicesSnapshot(snapshot())
            }

            is AppEvent.DevicesPatch -> {
                event.removed.forEach(devices::remove)
                event.added.forEach { devices[it.slotId] = it.toSnapshot() }
                WsEvent.DevicesPatch(
                    event.added.map { it.toSnapshot() },
                    event.removed
                )
            }

            is AppEvent.SlotsPatch -> {
                event.slots.forEach(::applySlot)
                WsEvent.SlotsPatch(event.slots.map { it.props })
            }

            is AppEvent.CustomAction -> WsEvent.CustomAction(event.action)
            is AppEvent.Unknown -> null
        }

    private fun applySlot(entry: SlotPatchEntry) {
        val current = devices[entry.slotId] ?: return
        devices[entry.slotId] = current.copy(
            props = mergeJson(current.props, entry.props),
            slotState = mergeJson(current.slotState, entry.slotState),
        )
    }

    private fun RemoteDevice.toSnapshot(): RemoteDeviceSnapshot =
        RemoteDeviceSnapshot(
            id = id,
            slotId = slotId,
            name = name,
            deviceType = deviceType,
            props = props,
            slotState = slotState,
        )

}
