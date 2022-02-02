use std::io;

use radix_engine::model::*;
use radix_engine::transaction::*;
use sbor::*;

use crate::ledger::*;
use crate::utils::*;

/// Represents a resim error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,

    HomeDirUnknown,

    ConfigDecodingError(sbor::DecodeError),

    IOError(io::Error),

    DataError(DecodeError),

    JSONError(serde_json::Error),

    CargoError(CargoExecutionError),

    TransactionConstructionError(BuildTransactionError),

    TransactionValidationError(TransactionValidationError),

    TransactionExecutionError(RuntimeError),

    LedgerDumpError(DisplayError),

    CompileError(transaction_manifest::CompileError),
}
