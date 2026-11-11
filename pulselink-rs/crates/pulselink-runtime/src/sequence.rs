//! B0 sequence allocation.

use pulselink_protocol::coyote::v3::Sequence;

/// Allocates intensity request sequence numbers in `1..=15`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SequenceAllocator {
    next: u8,
}

impl SequenceAllocator {
    pub(crate) fn new() -> Self {
        Self { next: 1 }
    }

    pub(crate) fn next(&mut self) -> Sequence {
        if self.next == 0 {
            self.next = 1;
        }
        let value = self.next;
        self.next = if value >= Sequence::MAX { 1 } else { value + 1 };
        Sequence::new(value).unwrap_or(Sequence::ZERO)
    }

    pub(crate) fn reset(&mut self) {
        self.next = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_without_zero() {
        let mut allocator = SequenceAllocator::new();
        let values: Vec<_> = (0..16).map(|_| allocator.next().get()).collect();
        assert_eq!(
            values,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1]
        );
        allocator.reset();
        assert_eq!(allocator.next().get(), 1);
    }
}
