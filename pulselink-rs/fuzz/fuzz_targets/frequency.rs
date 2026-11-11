#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_pattern::PulseInterval;
use pulselink_protocol::coyote::v3::{decode_wire_frequency, encode_interval};

fuzz_target!(|data: &[u8]| {
    let Some((&wire, rest)) = data.split_first() else {
        return;
    };
    if let Ok(interval) = decode_wire_frequency(wire) {
        let _ = encode_interval(interval);
    }
    if let Some(&raw) = rest.first() {
        if let Ok(interval) = PulseInterval::new(10 + u16::from(raw) % 991) {
            assert!(encode_interval(interval) >= 10);
        }
    }
});
