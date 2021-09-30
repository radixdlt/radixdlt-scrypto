pub use std::io;

pub use radix_engine::engine::*;
use radix_engine::transaction::*;
pub use sbor::*;
pub use scrypto::types::*;

use crate::utils::*;

#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,

    NoHomeFolder,

    MissingSubCommand,

    MissingArgument(String),

    InvalidAddress(ParseAddressError),

    InvalidAmount,

    IOError(io::Error),

    JSONError(serde_json::Error),

    CargoError(CargoExecutionError),

    TxnConstructionErr(BuildTransactionError),

    TxnExecutionError(RuntimeError),

    TransactionFailed,

    DataError(DecodeError),
}
