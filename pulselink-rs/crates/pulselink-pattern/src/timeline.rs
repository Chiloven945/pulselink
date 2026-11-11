//! A non-empty sequence of 100 ms frames.

use std::time::Duration;

use crate::error::PatternError;
use crate::frame::StereoFrame;

/// The duration of one frame.
pub const FRAME_DURATION: Duration = Duration::from_millis(100);

/// The duration of one frame in milliseconds.
pub const FRAME_DURATION_MS: u64 = 100;

/// A non-empty sequence of stereo frames.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timeline {
    frames: Vec<StereoFrame>,
}

impl Timeline {
    /// Creates a timeline from frames.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::Empty`] when `frames` is empty.
    pub fn new(frames: Vec<StereoFrame>) -> Result<Self, PatternError> {
        if frames.is_empty() {
            return Err(PatternError::Empty);
        }

        Ok(Self { frames })
    }

    /// Returns the number of frames.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns whether the timeline is empty.
    ///
    /// A valid timeline is never empty; this method exists for API completeness.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Returns the frames.
    #[must_use]
    pub fn frames(&self) -> &[StereoFrame] {
        &self.frames
    }

    /// Returns the frame at an index.
    #[must_use]
    pub fn frame(&self, index: usize) -> Option<&StereoFrame> {
        self.frames.get(index)
    }

    /// Returns the frame at a 100 ms frame index derived from elapsed time.
    #[must_use]
    pub fn frame_at_elapsed(&self, elapsed: Duration) -> Option<&StereoFrame> {
        let index = frame_index(elapsed)?;
        self.frame(index)
    }
}

/// Converts elapsed time to a 100 ms frame index.
#[must_use]
pub(crate) fn frame_index(elapsed: Duration) -> Option<usize> {
    let raw = elapsed.as_millis() / u128::from(FRAME_DURATION_MS);
    usize::try_from(raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::ChannelFrame;

    #[test]
    fn empty_timeline_is_rejected() {
        assert_eq!(Timeline::new(Vec::new()), Err(PatternError::Empty));
    }

    #[test]
    fn frame_lookup() {
        let timeline = Timeline::new(vec![
            StereoFrame::new(ChannelFrame::Silent, ChannelFrame::Silent),
            StereoFrame::new(ChannelFrame::Silent, ChannelFrame::Silent),
        ])
        .expect("valid timeline");

        assert_eq!(timeline.len(), 2);
        assert!(!timeline.is_empty());
        assert!(timeline.frame(0).is_some());
        assert!(timeline.frame(2).is_none());
        assert!(
            timeline
                .frame_at_elapsed(Duration::from_millis(150))
                .is_some()
        );
        assert!(
            timeline
                .frame_at_elapsed(Duration::from_millis(250))
                .is_none()
        );
    }
}
