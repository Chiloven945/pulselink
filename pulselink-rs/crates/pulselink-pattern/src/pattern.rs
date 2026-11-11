//! Patterns and playback cursors.

use std::time::Duration;

use crate::error::PatternError;
use crate::frame::StereoFrame;
use crate::timeline::{Timeline, frame_index};

/// A sequence of stereo frames with an optional loop.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pattern {
    timeline: Timeline,
    looped: bool,
}

impl Pattern {
    /// Creates a pattern from frames.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::Empty`] when `frames` is empty.
    pub fn new(frames: Vec<StereoFrame>, looped: bool) -> Result<Self, PatternError> {
        Ok(Self {
            timeline: Timeline::new(frames)?,
            looped,
        })
    }

    /// Creates a pattern from a timeline.
    #[must_use]
    pub const fn from_timeline(timeline: Timeline, looped: bool) -> Self {
        Self { timeline, looped }
    }

    /// Returns the underlying timeline.
    #[must_use]
    pub const fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Returns the number of frames.
    #[must_use]
    pub fn len(&self) -> usize {
        self.timeline.len()
    }

    /// Returns whether the pattern is empty.
    ///
    /// A valid pattern is never empty; this method exists for API completeness.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.timeline.is_empty()
    }

    /// Returns whether the pattern loops.
    #[must_use]
    pub const fn looped(&self) -> bool {
        self.looped
    }

    /// Returns the frame at an index, without applying the loop.
    #[must_use]
    pub fn frame_at_index(&self, index: usize) -> Option<&StereoFrame> {
        self.timeline.frame(index)
    }

    /// Returns the frame that should be played at `elapsed`, applying the loop.
    #[must_use]
    pub fn frame_at(&self, elapsed: Duration) -> Option<&StereoFrame> {
        let index = frame_index(elapsed)?;
        if self.looped {
            let index = index % self.timeline.len();
            self.timeline.frame(index)
        } else {
            self.timeline.frame(index)
        }
    }
}

/// A playback cursor over a [`Pattern`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatternCursor {
    elapsed: Duration,
    frame_index: Option<usize>,
}

impl Default for PatternCursor {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternCursor {
    /// Creates a cursor at the beginning of a pattern.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            frame_index: Some(0),
        }
    }

    /// Resets the cursor to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.frame_index = Some(0);
    }

    /// Returns the elapsed playback time.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the current frame index, if the pattern has not ended.
    #[must_use]
    pub const fn frame_index(&self) -> Option<usize> {
        self.frame_index
    }

    /// Advances the cursor by `delta` and returns the new frame index.
    ///
    /// Returns `None` when a non-looping pattern has ended.
    pub fn advance(&mut self, pattern: &Pattern, delta: Duration) -> Option<usize> {
        self.elapsed = self.elapsed.saturating_add(delta);
        let index = frame_index(self.elapsed)?;

        let resolved = if pattern.looped() {
            Some(index % pattern.len())
        } else if index < pattern.len() {
            Some(index)
        } else {
            None
        };

        self.frame_index = resolved;
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::ChannelFrame;

    fn pattern(looped: bool) -> Pattern {
        Pattern::new(
            vec![
                StereoFrame::new(ChannelFrame::Silent, ChannelFrame::Silent),
                StereoFrame::new(ChannelFrame::Silent, ChannelFrame::Silent),
            ],
            looped,
        )
        .expect("valid pattern")
    }

    #[test]
    fn empty_pattern_is_rejected() {
        assert_eq!(Pattern::new(Vec::new(), false), Err(PatternError::Empty));
    }

    #[test]
    fn non_looping_pattern_ends() {
        let pattern = pattern(false);
        assert!(pattern.frame_at(Duration::from_millis(50)).is_some());
        assert!(pattern.frame_at(Duration::from_millis(150)).is_some());
        assert!(pattern.frame_at(Duration::from_millis(250)).is_none());
    }

    #[test]
    fn looping_pattern_wraps() {
        let pattern = pattern(true);
        assert!(pattern.frame_at(Duration::from_millis(250)).is_some());
        assert_eq!(
            pattern.frame_at(Duration::from_millis(250)),
            pattern.frame_at_index(1)
        );
    }

    #[test]
    fn cursor_default_starts_at_beginning() {
        let cursor = PatternCursor::default();
        assert_eq!(cursor.frame_index(), Some(0));
        assert_eq!(cursor.elapsed(), Duration::ZERO);
    }

    #[test]
    fn cursor_advances() {
        let pattern = pattern(false);
        let mut cursor = PatternCursor::new();
        assert_eq!(cursor.frame_index(), Some(0));
        assert_eq!(
            cursor.advance(&pattern, Duration::from_millis(100)),
            Some(1)
        );
        assert_eq!(cursor.advance(&pattern, Duration::from_millis(100)), None);
        assert_eq!(cursor.frame_index(), None);
        cursor.reset();
        assert_eq!(cursor.frame_index(), Some(0));
        assert_eq!(cursor.elapsed(), Duration::ZERO);
    }

    #[test]
    fn looping_cursor_never_ends() {
        let pattern = pattern(true);
        let mut cursor = PatternCursor::new();
        assert_eq!(
            cursor.advance(&pattern, Duration::from_millis(300)),
            Some(1)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn pattern_serde_roundtrip() {
        let pattern = pattern(true);
        let json = serde_json::to_string(&pattern).expect("serialize");
        let roundtrip: Pattern = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, pattern);
    }
}
