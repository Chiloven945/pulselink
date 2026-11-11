#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_protocol::socket::v4::AppMessage;

fuzz_target!(|data: &[u8]| {
    if let Ok(message) = serde_json::from_slice::<AppMessage>(data) {
        let _ = serde_json::to_vec(&message);
    }
});
