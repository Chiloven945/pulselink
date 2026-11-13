package top.chiloven.pulselink.java;

import top.chiloven.pulselink.java.internal.BridgeContext;

final class BridgeFactory {

    private BridgeFactory() {
    }

    static PulseLinkBackend createClient() {
        return new PulseLinkBackendAdapter(new BridgeContext());
    }

}
