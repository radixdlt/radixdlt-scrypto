use std::io;

use radix_engine::engine::*;
use radix_engine::model::ExtractAbiError;
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

    PackageError(ExtractAbiError),

    TransactionConstructionError(CallWithAbiError),

    TransactionValidationError(TransactionValidationError),

    TransactionExecutionError(RuntimeError),

    AbiExportError(RuntimeError),

    LedgerDumpError(DisplayError),

    CompileError(transaction_manifest::CompileError),

    DecompileError(transaction_manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,
}
