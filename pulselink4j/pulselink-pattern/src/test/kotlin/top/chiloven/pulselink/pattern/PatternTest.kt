package top.chiloven.pulselink.pattern

import kotlin.test.*
import kotlin.time.Duration.Companion.milliseconds

class PatternTest {

    @Test
    fun intervalBoundsAndHz() {
        assertEquals(10, PulseInterval.of(10).milliseconds)
        assertEquals(1000, PulseInterval.of(1000).milliseconds)
        assertFailsWith<PatternException.InvalidInterval> { PulseInterval.of(9) }
        assertEquals(10, PulseInterval.fromHz(100.0).milliseconds)
        assertEquals(
            10.0,
            PulseInterval.of(100).toHz(),
            0.0001
        )
        assertFailsWith<PatternException.InvalidFrequency> { PulseInterval.fromHz(0.0) }
    }

    @Test
    fun pulseIntensityBounds() {
        assertEquals(100, PulseIntensity.MAX.value)
        assertFailsWith<PatternException.InvalidPulseIntensity> { PulseIntensity.of(101) }
    }

    @Test
    fun frameRequiresFourSamples() {
        val sample = PulseSample(
            PulseInterval.of(100),
            PulseIntensity.of(20)
        )
        assertFailsWith<PatternException.InvalidSampleCount> {
            ChannelFrame.Active(listOf(sample))
        }
        assertEquals(
            4,
            ChannelFrame.Active(
                listOf(
                    sample,
                    sample,
                    sample,
                    sample
                )
            ).samples.size
        )
    }

    @Test
    fun patternCursorLoopsAndEnds() {
        val frame = StereoFrame.silent()
        val timeline = Timeline.of(listOf(frame, frame))
        val once = PatternCursor(Pattern.of(timeline))
        assertEquals(0, once.frameIndex)
        assertTrue(once.advance())
        assertEquals(1, once.frameIndex)
        assertFalse(once.advance())
        assertTrue(once.isFinished)

        val looped = PatternCursor(Pattern.of(timeline, looped = true))
        assertTrue(looped.advance())
        assertTrue(looped.advance())
        assertEquals(0, looped.frameIndex)
    }

    @Test
    fun timelineLookup() {
        val frame = StereoFrame.silent()
        val timeline = Timeline.of(listOf(frame, frame))
        assertEquals(2, timeline.size)
        assertSame(frame, timeline.frame(0))
        assertNull(timeline.frame(2))
        assertSame(frame, timeline.frameAtElapsed(150.milliseconds))
        assertNull(timeline.frameAtElapsed(250.milliseconds))
        assertFailsWith<PatternException.EmptyTimeline> { Timeline.of(emptyList()) }
    }

}
