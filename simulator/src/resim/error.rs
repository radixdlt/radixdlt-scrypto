use std::fmt;
use std::io;
use std::path::PathBuf;

use radix_engine::errors::{RejectionReason, RuntimeError};
use radix_engine::transaction::AbortReason;
use radix_engine::utils::ExtractSchemaError;
use radix_engine::vm::wasm::PrepareError;
use radix_engine_common::network::ParseNetworkError;
use radix_engine_common::prelude::{ComponentAddress, NodeId, PackageAddress};
use radix_engine_interface::types::ParseNonFungibleGlobalIdError;
use radix_engine_interface::types::SchemaHash;
use sbor::*;
use transaction::errors::*;
use transaction::model::PrepareError as TransactionPrepareError;

use crate::resim::EntityDumpError;
use crate::utils::*;

/// Represents a resim error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,
    NoDefaultPrivateKey,
    NoDefaultOwnerBadge,

    HomeDirUnknown,

    PackageNotFound(PackageAddress),
    SchemaNotFound(NodeId, SchemaHash),
    BlueprintNotFound(PackageAddress, String),
    ComponentNotFound(ComponentAddress),
    InstanceSchemaNot(ComponentAddress, u8),

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    SborDecodeError(DecodeError),

    SborEncodeError(EncodeError),

    BuildError(BuildError),

    ExtractSchemaError(ExtractSchemaError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallInstructionError),

    TransactionValidationError(TransactionValidationError),

    TransactionPrepareError(TransactionPrepareError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionReason),

    TransactionAborted(AbortReason),

    LedgerDumpError(EntityDumpError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    /// e.g. if you accidentally pass in a public key in `set_default_account` command.
    GotPublicKeyExpectedPrivateKey,

    NonFungibleGlobalIdError(ParseNonFungibleGlobalIdError),

    FailedToBuildArguments(BuildCallArgumentError),

    ParseNetworkError(ParseNetworkError),

    OwnerBadgeNotSpecified,

    InstructionSchemaValidationError(radix_engine::utils::LocatedInstructionSchemaValidationError),

    InvalidResourceSpecifier(String),

    RemoteGenericSubstitutionNotSupported,
}

impl fmt::Display for Error {
    // TODO Implement pretty error printing
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        err.to_string()
    }
}
