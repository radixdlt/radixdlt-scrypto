use std::io;
use std::path::PathBuf;

use radix_engine::engine::*;
use radix_engine::model::{ExportError, ExtractAbiError};
use radix_engine::types::{AddressError, ParseNonFungibleAddressError};
use radix_engine::wasm::PrepareError;
use radix_engine_interface::node::ParseNetworkError;
use sbor::*;
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

    IOErrorAtPath(io::Error, PathBuf),

    DataError(DecodeError),

    JSONError(serde_json::Error),

    BuildError(BuildError),

    PackageAddressNotFound,

    ExtractAbiError(ExtractAbiError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallWithAbiError),

    TransactionValidationError(TransactionValidationError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionError),

    AbiExportError(ExportError),

    LedgerDumpError(DisplayError),

    CompileError(transaction::manifest::CompileError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    AddressError(AddressError),

    NonFungibleAddressError(ParseNonFungibleAddressError),

    FailedToBuildArgs(BuildArgsError),

    ParseNetworkError(ParseNetworkError),

    OwnerBadgeNotSpecified,
}
