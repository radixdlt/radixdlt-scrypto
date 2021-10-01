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

    InvalidAmount(ParseAmountError),

    IOError(io::Error),

    ConfigDecodeError(sbor::DecodeError),

    CargoError(CargoExecutionError),

    TransactionConstructionError(BuildTransactionError),

    TransactionExecutionError(RuntimeError),

    TransactionFailed,

    DataError(DecodeError),

    JSONError(serde_json::Error),
}
