use flatbuffers::InvalidFlatbuffer;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Malformed(&'static str),
    IO(std::io::Error),
    InvalidFlatbuffer(InvalidFlatbuffer),
    #[cfg(feature = "http")]
    HttpClient(http_range_client::HttpError),
}
pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Malformed(description) => description.fmt(f),
            Error::IO(io) => io.fmt(f),
            Error::InvalidFlatbuffer(invalid_flatbuffer) => invalid_flatbuffer.fmt(f),
            #[cfg(feature = "http")]
            Error::HttpClient(http_client) => http_client.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<InvalidFlatbuffer> for Error {
    fn from(value: InvalidFlatbuffer) -> Self {
        Error::InvalidFlatbuffer(value)
    }
}

#[cfg(feature = "http")]
impl From<http_range_client::HttpError> for Error {
    fn from(value: http_range_client::HttpError) -> Self {
        Error::HttpClient(value)
    }
}
