use crate::internal_prelude::*;

#[derive(Debug)]
pub enum Error {
    ParseNetworkError(ParseNetworkError),
    IOError(std::io::Error),
    DatabaseError(rocksdb::Error),
    InvalidTransactionArchive,
    InvalidTransactionSource,
    InvalidBreakpoints(String),
}
