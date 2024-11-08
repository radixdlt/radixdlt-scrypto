use crate::prelude::*;
use crate::resim::EntityDumpError;
use radix_engine::errors::*;
use radix_engine::transaction::AbortReason;
use radix_engine::vm::wasm::PrepareError as WasmPrepareError;
use radix_transactions::errors::*;
use radix_transactions::manifest::DecompileError;
use radix_transactions::model::PrepareError as TransactionPrepareError;
use std::io;

/// Represents a resim error.
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

    InvalidPackage(WasmPrepareError),

    TransactionConstructionError(BuildCallInstructionError),

    TransactionValidationError(TransactionValidationError),

    TransactionPrepareError(TransactionPrepareError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionReason),

    TransactionAborted(AbortReason),

    LedgerDumpError(EntityDumpError),

    DecompileError(DecompileError),

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

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let address_encoder = AddressBech32Encoder::for_simulator();
        match self {
            Error::PackageNotFound(package_address) => {
                write!(
                    f,
                    "PackageNotFound({})",
                    package_address.display(&address_encoder)
                )
            }
            Error::SchemaNotFound(node_id, schema_hash) => {
                write!(
                    f,
                    "SchemaNotFound({}, {schema_hash:?})",
                    node_id.display(&address_encoder)
                )
            }
            Error::BlueprintNotFound(package_address, message) => {
                write!(
                    f,
                    "BlueprintNotFound({}, {message})",
                    package_address.display(&address_encoder)
                )
            }
            Error::ComponentNotFound(component_address) => {
                write!(
                    f,
                    "ComponentNotFound({})",
                    component_address.display(&address_encoder)
                )
            }
            Error::InstanceSchemaNot(component_address, index) => {
                write!(
                    f,
                    "InstanceSchemaNot({}, {index})",
                    component_address.display(&address_encoder)
                )
            }
            Error::TransactionFailed(runtime_error) => {
                write!(
                    f,
                    "TransactionFailed({})",
                    runtime_error.display(&address_encoder)
                )
            }
            Error::TransactionRejected(rejection_reason) => {
                write!(
                    f,
                    "TransactionRejected({})",
                    rejection_reason.display(&address_encoder)
                )
            }
            other => write!(f, "{:?}", other),
        }
    }
}
