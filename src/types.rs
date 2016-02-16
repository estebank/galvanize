use std::io::Error as IOError;
use std::result;
use std::error;
use std::fmt;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    CDBTooSmall,
    KeyNotInCDB,
    IOError(IOError),
}

pub type Result<T> = result::Result<T, Error>;


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CDBTooSmall => write!(f, "File too small to be a CDB"),
            Error::KeyNotInCDB => write!(f, "The key is not in the CDB"),
            Error::IOError(ref e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            Error::CDBTooSmall => "The file is too small to be a valid CDB",
            Error::KeyNotInCDB => "The key is not in the CDB",
            Error::IOError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::CDBTooSmall => None,
            Error::KeyNotInCDB => None,
            Error::IOError(ref e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IOError(e)
    }
}
