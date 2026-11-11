#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_protocol::coyote::v3::{B1Message, decode_battery};

fuzz_target!(|data: &[u8]| {
    let _ = B1Message::decode(data);
    let _ = decode_battery(data);
});
