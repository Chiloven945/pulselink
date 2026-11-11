//! Safety policy checks.

use pulselink_core::{Channel, Intensity, IntensityDelta};

use crate::config::SafetyPolicy;
use crate::error::RuntimeError;

pub(crate) fn effective_limit(
    policy: &SafetyPolicy,
    channel: Channel,
    soft_limit: Option<Intensity>,
) -> Intensity {
    let policy_limit = *policy.max_intensity.get(channel);
    soft_limit.map_or(policy_limit, |limit| limit.min(policy_limit))
}

pub(crate) fn check_absolute(
    policy: &SafetyPolicy,
    channel: Channel,
    target: Intensity,
    soft_limit: Option<Intensity>,
) -> Result<(), RuntimeError> {
    let limit = effective_limit(policy, channel, soft_limit);
    if target > limit {
        return Err(RuntimeError::SafetyLimit(format!(
            "channel {channel} intensity {} exceeds limit {}",
            target.get(),
            limit.get()
        )));
    }
    Ok(())
}

pub(crate) fn check_delta(
    policy: &SafetyPolicy,
    channel: Channel,
    delta: IntensityDelta,
) -> Result<(), RuntimeError> {
    if let Some(max_delta) = policy.max_delta {
        let limit = max_delta.get(channel).magnitude();
        if delta.magnitude() > limit {
            return Err(RuntimeError::SafetyLimit(format!(
                "channel {channel} delta {} exceeds limit {}",
                delta.get(),
                limit
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulselink_core::ChannelMap;

    fn policy() -> SafetyPolicy {
        SafetyPolicy {
            max_intensity: ChannelMap::new(
                Intensity::new(100).expect("valid"),
                Intensity::new(80).expect("valid"),
            ),
            max_delta: Some(ChannelMap::new(
                IntensityDelta::new(10).expect("valid"),
                IntensityDelta::new(20).expect("valid"),
            )),
            ..SafetyPolicy::default()
        }
    }

    #[test]
    fn absolute_and_soft_limits_use_stricter_value() {
        let policy = policy();
        let soft = Some(Intensity::new(50).expect("valid"));
        assert!(
            check_absolute(
                &policy,
                Channel::A,
                Intensity::new(50).expect("valid"),
                soft
            )
            .is_ok()
        );
        assert!(
            check_absolute(
                &policy,
                Channel::A,
                Intensity::new(51).expect("valid"),
                soft
            )
            .is_err()
        );
    }

    #[test]
    fn delta_limits_apply_per_channel() {
        let policy = policy();
        assert!(check_delta(&policy, Channel::A, IntensityDelta::new(10).expect("valid")).is_ok());
        assert!(check_delta(&policy, Channel::A, IntensityDelta::new(11).expect("valid")).is_err());
        assert!(
            check_delta(
                &policy,
                Channel::B,
                IntensityDelta::new(-21).expect("valid")
            )
            .is_err()
        );
    }
}
