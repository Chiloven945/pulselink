package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.device.LocalSession
import top.chiloven.pulselink.device.LocalSession as KotlinLocalSession

internal fun KotlinLocalSession.toJava(): LocalSession =
    when (this) {
        is KotlinLocalSession.CoyoteV2 -> CoyoteV2SessionAdapter(BridgeContext(), this)
        is KotlinLocalSession.CoyoteV3 -> CoyoteV3SessionAdapter(BridgeContext(), runtime)
        is KotlinLocalSession.Opossum -> OpossumSessionAdapter(BridgeContext(), this)
        is KotlinLocalSession.Civet -> CivetSessionAdapter(BridgeContext(), this)
        is KotlinLocalSession.PawPrints -> PawPrintsSessionAdapter(BridgeContext(), this)
    }
