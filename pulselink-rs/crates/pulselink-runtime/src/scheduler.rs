//! Scheduler timing helpers.

use std::time::Duration;

/// Returns the 100 ms frame index for an elapsed duration.
pub(crate) fn frame_index(elapsed: Duration, frame_duration: Duration) -> Option<usize> {
    if frame_duration.is_zero() {
        return None;
    }
    let raw = elapsed.as_millis() / frame_duration.as_millis().max(1);
    usize::try_from(raw).ok()
}

/// Returns whether the scheduler is later than the configured threshold.
pub(crate) fn is_lagging(elapsed: Duration, threshold: Duration) -> bool {
    elapsed > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_index_uses_elapsed_time() {
        assert_eq!(
            frame_index(Duration::from_millis(99), Duration::from_millis(100)),
            Some(0)
        );
        assert_eq!(
            frame_index(Duration::from_millis(250), Duration::from_millis(100)),
            Some(2)
        );
        assert_eq!(frame_index(Duration::ZERO, Duration::ZERO), None);
    }

    #[test]
    fn lag_is_strictly_greater_than_threshold() {
        let threshold = Duration::from_millis(500);
        assert!(!is_lagging(Duration::from_millis(500), threshold));
        assert!(is_lagging(Duration::from_millis(501), threshold));
    }
}
