use std::io;

pub type Result<T> = std::result::Result<T, Error>;

/// Define an enumeration for the various errors that can occur in this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("End of buffer")]
    EndOfBuffer,
    #[error("Limit of {0} jumps exceeded")]
    JumpLimit(usize),
    #[error("Single label exceeds 63 characters of length")]
    SingleLabelLimit,
    #[error("{0}")]
    Io(#[from] io::Error),
}
