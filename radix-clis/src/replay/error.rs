use radix_engine_common::prelude::ParseNetworkError;

#[derive(Debug)]
pub enum Error {
    ParseNetworkError(ParseNetworkError),
    IOError(std::io::Error),
    DatabaseError(rocksdb::Error),
    InvalidTransactionArchive,
    InvalidTransactionSource,
    InvalidBreakpoints(String),
}
