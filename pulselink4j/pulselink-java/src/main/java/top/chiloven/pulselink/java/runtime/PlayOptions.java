package top.chiloven.pulselink.java.runtime;

/** Options for starting pattern playback. */
public record PlayOptions(boolean replace) {

    public static PlayOptions defaults() {
        return new PlayOptions(false);
    }

}
