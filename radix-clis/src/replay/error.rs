use radix_common::prelude::ParseNetworkError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParseNetworkError(ParseNetworkError),
    IOError(std::io::Error),
    DatabaseError(rocksdb::Error),
    InvalidTransactionArchive,
    InvalidTransactionSource,
    InvalidBreakpoints(String),
}

impl fmt::Display for Error {
    // TODO Implement pretty error printing
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        err.to_string()
    }
}
