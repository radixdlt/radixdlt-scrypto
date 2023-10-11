use radix_engine_interface::prelude::ParseNetworkError;

#[derive(Debug)]
pub enum Error {
    ParseNetworkError(ParseNetworkError),
    IOError(std::io::Error),
    DatabaseError(rocksdb::Error),
}
