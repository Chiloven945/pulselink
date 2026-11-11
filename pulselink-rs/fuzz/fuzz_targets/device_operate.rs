#![no_main]

use libfuzzer_sys::fuzz_target;
use pulselink_protocol::socket::v4::DeviceOperate;

fuzz_target!(|data: &[u8]| {
    if let Ok(operate) = serde_json::from_slice::<DeviceOperate>(data) {
        let _ = operate.validate();
        let _ = serde_json::to_vec(&operate);
    }
});
