use crate::dns;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    DNS(#[from] dns::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("In-memory mode")]
    SaveButInMemory,
}
