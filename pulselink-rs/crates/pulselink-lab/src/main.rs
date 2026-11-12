//! `pulselink-lab`: terminal test bench for all supported devices.

use pulselink_lab::app::LabApp;
use pulselink_lab::worker::LabWorker;

fn main() -> std::io::Result<()> {
    LabApp::new(LabWorker::spawn()).run()
}
