//! This module contains the possible responses from the public interfaces in
//! this crate.
//!
//! [`Result<T>`](type.Result.html) can be either `T` or an
//! [`Error`](enum.Error.html).
use std::io::Error as IOError;
use std::result;
use std::error;
use std::fmt;
use std::convert::From;

/// An error in the interaction with the CDB.
#[derive(Debug)]
pub enum Error {
    /// The CDB is under 2048 bytes. The file being read is not a valid CDB.
    CDBTooSmall,
    /// The `key` being fetched isn't in the CDB.
    KeyNotInCDB,
    /// There was an error accessing the file.  It wraps the original
    /// `std::io::Error`.
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
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            Error::CDBTooSmall => "The file is too small to be a valid CDB",
            Error::KeyNotInCDB => "The key is not in the CDB",
            // The underlying error already impl `Error`, so we defer to its 
            // implementation.
            Error::IOError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::CDBTooSmall => None,
            Error::KeyNotInCDB => None,
            Error::IOError(ref e) => Some(e),
        }
    }
}

/// Allows seamless conversion from a `galvanize::Error` into an
/// `std::io::Error`. This way, the `try!()` macro can be used directly.
impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IOError(e)
    }
}
