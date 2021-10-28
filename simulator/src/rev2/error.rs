use std::io;
use std::num::ParseIntError;

use radix_engine::engine::*;
use radix_engine::transaction::*;
use sbor::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::utils::*;

/// Represents a REv2 error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,

    MissingHomeDirectory,

    MissingSubCommand,

    MissingArgument(String),

    InvalidAddress(ParseAddressError),

    InvalidAmount(ParseAmountError),

    InvalidNumber(ParseIntError),

    InvalidConfig(sbor::DecodeError),

    InvalidSignerPublicKey,

    IOError(io::Error),

    DataError(DecodeError),

    JSONError(serde_json::Error),

    CargoError(CargoExecutionError),

    TransactionConstructionError(BuildTransactionError),

    TransactionExecutionError(RuntimeError),

    LedgerDumpError(DisplayError),

    TransactionFailed,
}
