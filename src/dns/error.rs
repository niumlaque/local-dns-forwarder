use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    EndOfBuffer,
    JumpLimit(usize),
    SingleLabelLimit,
    Io(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfBuffer => write!(f, "End of buffer"),
            Self::JumpLimit(v) => write!(f, "Limit of {} jumps exceeded", v),
            Self::SingleLabelLimit => write!(f, "Single label exceeds 63 characters of length"),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl std::error::Error for Error {}
