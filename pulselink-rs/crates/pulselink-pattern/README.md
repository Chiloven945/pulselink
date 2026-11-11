# pulselink-pattern

Device-agnostic waveform timeline types for pulselink-rs.

- `PulseInterval` — 10..=1000 ms; the official "pulse frequency value" is in ms.
- `PulseIntensity` — 0..=100.
- `PulseSample` — one 25 ms window.
- `ChannelFrame` — four samples (100 ms), or explicit silence.
- `StereoFrame` — A/B channels.
- `Timeline` — non-empty frame sequence.
- `Pattern` / `PatternCursor` — playback with optional looping.

```rust
use pulselink_pattern::{
    ChannelFrame, Pattern, PulseIntensity, PulseInterval, PulseSample, StereoFrame,
};

let sample = PulseSample::new(
PulseInterval::new(100) ?,
PulseIntensity::new(20) ?,
);
let frame = StereoFrame::new(ChannelFrame::active([sample; 4]), ChannelFrame::Silent);
let pattern = Pattern::new(vec![frame], false) ?;
assert_eq!(pattern.len(), 1);
# Ok::<(), pulselink_pattern::PatternError>(())
```

This crate does not encode Coyote `B0` bytes. That belongs to
`pulselink-protocol`. See `docs/ROADMAP.md` for the project plan.
