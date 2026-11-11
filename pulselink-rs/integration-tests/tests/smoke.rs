//! A smoke test for the M0/M1 workspace foundation.

use pulselink_core::{Channel, ChannelMap, Intensity, Observed};
use pulselink_pattern::{
    ChannelFrame, Pattern, PulseIntensity, PulseInterval, PulseSample, StereoFrame,
};

#[test]
fn core_and_pattern_are_usable_together() {
    let intensity = Intensity::new(25).expect("valid intensity");
    let limits = ChannelMap::new(intensity, Intensity::MIN);
    assert_eq!(limits[Channel::A], intensity);

    let observed: Observed<Intensity> = Observed::Known(intensity);
    assert_eq!(observed.copied(), Some(intensity));

    let sample = PulseSample::new(
        PulseInterval::new(50).expect("valid interval"),
        PulseIntensity::new(30).expect("valid intensity"),
    );
    let frame = StereoFrame::new(ChannelFrame::active([sample; 4]), ChannelFrame::Silent);
    let pattern = Pattern::new(vec![frame], false).expect("valid pattern");
    assert_eq!(pattern.len(), 1);
}
