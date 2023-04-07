use std::io;
use std::path::PathBuf;

use radix_engine::errors::{RejectionError, RuntimeError};
use radix_engine::transaction::AbortReason;
use radix_engine::types::{ComponentAddress, PackageAddress};
use radix_engine::utils::ExtractSchemaError;
use radix_engine::wasm::PrepareError;
use radix_engine_interface::blueprints::resource::ParseNonFungibleGlobalIdError;
use radix_engine_interface::network::ParseNetworkError;
use sbor::*;
use transaction::errors::*;

use crate::ledger::EntityDumpError;
use crate::utils::*;

/// Represents a resim error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,
    NoDefaultPrivateKey,
    NoDefaultOwnerBadge,

    HomeDirUnknown,

    PackageNotFound(PackageAddress),
    BlueprintNotFound(PackageAddress, String),
    ComponentNotFound(ComponentAddress),

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    SborDecodeError(DecodeError),

    SborEncodeError(EncodeError),

    BuildError(BuildError),

    ExtractSchemaError(ExtractSchemaError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallInstructionError),

    TransactionValidationError(TransactionValidationError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionError),

    TransactionAborted(AbortReason),

    LedgerDumpError(EntityDumpError),

    CompileError(transaction::manifest::CompileError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    NonFungibleGlobalIdError(ParseNonFungibleGlobalIdError),

    FailedToBuildArguments(BuildCallArgumentError),

    ParseNetworkError(ParseNetworkError),

    OwnerBadgeNotSpecified,
}
