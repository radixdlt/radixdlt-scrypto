use std::io;

use radix_engine::engine::*;
use radix_engine::model::ExtractAbiError;
use radix_engine::wasm::PrepareError;
use sbor::*;
use scrypto::address::AddressError;
use transaction::errors::*;

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

    BuildError(BuildError),

    PackageAddressNotFound,

    ExtractAbiError(ExtractAbiError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallWithAbiError),

    TransactionValidationError(TransactionValidationError),

    TransactionExecutionError(RuntimeError),

    TransactionRejected(RejectionError),

    AbiExportError(RuntimeError),

    LedgerDumpError(DisplayError),

    CompileError(transaction::manifest::CompileError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    AddressError(AddressError),

    FailedToBuildArgs(BuildArgsError),
}
