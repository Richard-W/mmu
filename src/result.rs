/// Result type for this crate
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for this crate
#[derive(Debug)]
pub enum Error {
    NoMemory,
    Overlap,
}
