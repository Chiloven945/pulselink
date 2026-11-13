package top.chiloven.pulselink.java.runtime;

/** Current V3 settings. */
public record CoyoteV3Settings(
        int softLimitA,
        int softLimitB,
        int frequencyBalanceA,
        int frequencyBalanceB,
        int intensityBalanceA,
        int intensityBalanceB
) {

}
