#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_protocol::socket::v4::RelayFrame;

fuzz_target!(|data: &[u8]| {
    if let Ok(frame) = serde_json::from_slice::<RelayFrame>(data) {
        let _ = serde_json::to_vec(&frame);
    }
});
