use std::fmt::Display;
use std::{fmt, io};
use std::time::Duration;
use crate::{MAX_ENTRY_LENGTH, MAX_PAYLOAD_LENGTH};

pub enum WhereError {
    EncodeDecodeError(EncodeDecodeError),
    IOError(io::Error),
    TimedOut(String, usize, Duration)
}

pub enum EncodeDecodeError {
    InvalidEntryLength(usize),
    InvalidPayloadLength(usize),
    BadMagic([u8; 4]),
    IncorrectEntryCount,
    StringSizeLimitExceeded(u32, usize),
    NonbinaryBoolean,
    EmptyRemote,
    IOErrorWhileTranscoding(io::Error)
}

pub type WhereResult<T> = Result<T, WhereError>;
pub type EncodeDecodeResult<T> = Result<T, EncodeDecodeError>;

impl From<io::Error> for WhereError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<EncodeDecodeError> for WhereError {
    fn from(value: EncodeDecodeError) -> Self {
        Self::EncodeDecodeError(value)
    }
}

impl From<io::Error> for EncodeDecodeError {
    fn from(value: io::Error) -> Self {
        Self::IOErrorWhileTranscoding(value)
    }
}

impl Display for EncodeDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntryLength(s) => write!(f, "Invalid entry length: {s} but maximum is {MAX_ENTRY_LENGTH}"),
            Self::InvalidPayloadLength(s) => write!(f, "Invalid full payload length: {s} but maximum is {MAX_PAYLOAD_LENGTH}"),
            Self::BadMagic(m) => write!(f, "Invalid packet magic ({}), possible corruption or invalid server", String::from_utf8_lossy(m)),
            Self::IncorrectEntryCount => write!(f, "Invalid amount of entries decoded"),
            Self::StringSizeLimitExceeded(curr, max) => write!(f, "Exceeded length limit for payload string ({curr} > {max})"),
            Self::NonbinaryBoolean => write!(f, "Boolean value is not 0 or 1"),
            Self::EmptyRemote => write!(f, "Remote tag set but no remote host is present"),
            Self::IOErrorWhileTranscoding(e) => write!(f, "Input/output error while encoding/decoding: {e}"),
        }
    }
}

impl Display for WhereError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EncodeDecodeError(e) => write!(f, "Encode/decode error: {e}"),
            Self::IOError(e) => write!(f, "Input/output error: {e}"),
            Self::TimedOut(server, max_retry, timeout) => write!(f, "Timed out waiting for data from {server} after {max_retry} attempts every {} ms", timeout.as_millis())
        }
    }
}