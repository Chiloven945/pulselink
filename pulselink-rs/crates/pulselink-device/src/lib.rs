//! Local DG-LAB device drivers.

mod civet;
mod client;
mod coyote_v2;
mod error;
mod opossum;
mod paw_prints;
mod profile;
mod session;

pub use civet::CivetSession;
pub use client::DeviceClient;
pub use coyote_v2::CoyoteV2Session;
pub use error::DeviceError;
pub use opossum::OpossumSession;
pub use paw_prints::{PawPrintsSession, PawPrintsSnapshot};
pub use profile::{
    CIVET, COYOTE_V2, COYOTE_V3, DeviceProfile, OPOSSUM, PAW_PRINTS, PROFILES, classify,
    profile_for,
};
pub use session::{LocalEvent, LocalSession};
