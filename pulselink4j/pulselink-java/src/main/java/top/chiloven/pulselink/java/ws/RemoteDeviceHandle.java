package top.chiloven.pulselink.java.ws;

import top.chiloven.pulselink.java.core.Channel;
import top.chiloven.pulselink.java.core.DeviceKind;

import java.time.Duration;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/** A handle for one remote device. */
public interface RemoteDeviceHandle {

    String slotId();

    Optional<RemoteDevice> snapshot();

    DeviceKind kind();

    Optional<TypedRemoteDevice> typed();

    default CompletableFuture<OperationHandle> resetIntensity(Channel channel) {
        return resetIntensity(channel, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> resetIntensity(
            Channel channel,
            OperationOptions options
    );

    default CompletableFuture<OperationHandle> addIntensity(Channel channel, int value) {
        return addIntensity(channel, value, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> addIntensity(
            Channel channel,
            int value,
            OperationOptions options
    );

    default CompletableFuture<OperationHandle> reduceIntensity(Channel channel, int value) {
        return reduceIntensity(channel, value, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> reduceIntensity(
            Channel channel,
            int value,
            OperationOptions options
    );

    default CompletableFuture<OperationHandle> setTempIntensity(
            Channel channel,
            int intensity,
            Duration duration
    ) {
        return setTempIntensity(channel, intensity, duration, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> setTempIntensity(
            Channel channel,
            int intensity,
            Duration duration,
            OperationOptions options
    );

    default CompletableFuture<OperationHandle> sendPulse(
            Channel channel,
            Duration duration,
            List<byte[]> frames
    ) {
        return sendPulse(channel, duration, frames, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> sendPulse(
            Channel channel,
            Duration duration,
            List<byte[]> frames,
            OperationOptions options
    );

    default CompletableFuture<OperationHandle> sendPulseV2(
            Channel channel,
            Duration duration,
            List<byte[]> frames
    ) {
        return sendPulseV2(channel, duration, frames, OperationOptions.defaults());
    }

    CompletableFuture<OperationHandle> sendPulseV2(
            Channel channel,
            Duration duration,
            List<byte[]> frames,
            OperationOptions options
    );

}
