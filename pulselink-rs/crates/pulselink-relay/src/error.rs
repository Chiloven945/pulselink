//! Relay errors.

/// An error returned by the local relay.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RelayError {
    /// The TCP listener could not be created.
    #[error("relay I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The relay task failed.
    #[error("relay task error: {0}")]
    Task(String),
}
