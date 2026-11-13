package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.Serializable

/** Incremental slot state patch. */
@Serializable
public data class SlotPatch(
    public val slots: List<SlotPatchEntry> = emptyList(),
)
