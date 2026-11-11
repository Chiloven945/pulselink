//! B0 conversion helpers.

use pulselink_core::Channel;
use pulselink_pattern::ChannelFrame;
use pulselink_protocol::coyote::v3::{ChannelWaveform, IntensityOp, StereoIntensityOp};

pub(super) fn keep_intensity() -> StereoIntensityOp {
    StereoIntensityOp {
        a: IntensityOp::Keep,
        b: IntensityOp::Keep,
    }
}

pub(super) fn op_for_channel(
    intensity: &mut StereoIntensityOp,
    channel: Channel,
) -> &mut IntensityOp {
    match channel {
        Channel::A => &mut intensity.a,
        Channel::B => &mut intensity.b,
    }
}

pub(super) fn waveform(frame: ChannelFrame) -> ChannelWaveform {
    match frame {
        ChannelFrame::Silent => ChannelWaveform::Silent,
        ChannelFrame::Active(samples) => ChannelWaveform::Active(samples),
    }
}
