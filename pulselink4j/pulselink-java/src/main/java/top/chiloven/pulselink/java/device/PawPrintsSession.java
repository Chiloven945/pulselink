package top.chiloven.pulselink.java.device;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/** A connected PawPrints session. */
public non-sealed interface PawPrintsSession extends LocalSession {

    CompletableFuture<Void> noTrigger(int color);

    CompletableFuture<Void> resetParameter();

    CompletableFuture<Void> autoDetectAngles();

    CompletableFuture<Void> shoulderLight(int color);

    CompletableFuture<Void> shoulderLightFlash(
            int colorA,
            int colorB,
            int speed
    );

    CompletableFuture<Void> configureRandomReaction(
            int color,
            int eventId,
            int delayMin,
            int delayMax,
            int reactionSeconds,
            int increaseRate,
            int decreaseOnDeactivation,
            int decreaseRate,
            int increaseOnActivation
    );

    CompletableFuture<Void> configureProbability(
            int color,
            List<PawPrintsProbability> events,
            int cooldownSeconds
    );

    CompletableFuture<Void> configureExternalVoltage(
            int color,
            int eventId,
            boolean highImpedance,
            int targetMin,
            int targetMax,
            int mappingRange
    );

    CompletableFuture<Void> configureRawSensor(int color);

}
