package top.chiloven.pulselink.java.ws;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Flow;

/** A connected DG-LAB app client. */
public interface AppClient {

    String clientId();

    List<RemoteDevice> devices();

    CompletableFuture<List<RemoteDevice>> requestDevices();

    Optional<RemoteDeviceHandle> device(String slotId);

    Flow.Publisher<WsEvent> events();

    CompletableFuture<Long> ping();

    CompletableFuture<Void> clearOperate(ClearTarget target);

    default CompletableFuture<OperationHandle> submitOperate(DeviceOperation operation) {
        return submitOperate(
                operation,
                OperationOptions.defaults()
        );
    }

    CompletableFuture<OperationHandle> submitOperate(
            DeviceOperation operation,
            OperationOptions options
    );

}
