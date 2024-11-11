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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address_encoder = AddressBech32Encoder::for_simulator();
        let address_encoder = &address_encoder;
        match self {
            // Overriden ones
            Self::PackageNotFound(package_address) => f
                .debug_tuple("PackageNotFound")
                .field(&package_address.to_string(address_encoder))
                .finish(),
            Self::SchemaNotFound(node_id, schema_hash) => f
                .debug_tuple("SchemaNotFound")
                .field(&node_id.to_string(address_encoder))
                .field(schema_hash)
                .finish(),
            Self::BlueprintNotFound(package_address, blueprint) => f
                .debug_tuple("BlueprintNotFound")
                .field(&package_address.to_string(address_encoder))
                .field(blueprint)
                .finish(),
            Self::ComponentNotFound(component_address) => f
                .debug_tuple("ComponentNotFound")
                .field(&component_address.to_string(address_encoder))
                .finish(),
            Self::InstanceSchemaNot(component_address, index) => f
                .debug_tuple("InstanceSchemaNot")
                .field(&component_address.to_string(address_encoder))
                .field(index)
                .finish(),
            Self::TransactionFailed(runtime_error) => f
                .debug_tuple("TransactionFailed")
                .field(&runtime_error.to_string(address_encoder))
                .finish(),
            Self::TransactionRejected(rejection_reason) => f
                .debug_tuple("TransactionRejected")
                .field(&rejection_reason.to_string(address_encoder))
                .finish(),
            // Automatic / Code-gen'd ones
            Self::NoDefaultAccount => write!(f, "NoDefaultAccount"),
            Self::NoDefaultPrivateKey => write!(f, "NoDefaultPrivateKey"),
            Self::NoDefaultOwnerBadge => write!(f, "NoDefaultOwnerBadge"),
            Self::HomeDirUnknown => write!(f, "HomeDirUnknown"),
            Self::IOError(err) => f.debug_tuple("IOError").field(err).finish(),
            Self::IOErrorAtPath(err, path) => f
                .debug_tuple("IOErrorAtPath")
                .field(err)
                .field(path)
                .finish(),
            Self::SborDecodeError(err) => f.debug_tuple("SborDecodeError").field(err).finish(),
            Self::SborEncodeError(err) => f.debug_tuple("SborEncodeError").field(err).finish(),
            Self::BuildError(err) => f.debug_tuple("BuildError").field(err).finish(),
            Self::ExtractSchemaError(err) => {
                f.debug_tuple("ExtractSchemaError").field(err).finish()
            }
            Self::InvalidPackage(err) => f.debug_tuple("InvalidPackage").field(err).finish(),
            Self::TransactionConstructionError(err) => f
                .debug_tuple("TransactionConstructionError")
                .field(err)
                .finish(),
            Self::TransactionValidationError(err) => f
                .debug_tuple("TransactionValidationError")
                .field(err)
                .finish(),
            Self::TransactionPrepareError(err) => {
                f.debug_tuple("TransactionPrepareError").field(err).finish()
            }
            Self::TransactionAborted(reason) => {
                f.debug_tuple("TransactionAborted").field(reason).finish()
            }
            Self::LedgerDumpError(err) => f.debug_tuple("LedgerDumpError").field(err).finish(),
            Self::DecompileError(err) => f.debug_tuple("DecompileError").field(err).finish(),
            Self::InvalidId(id) => f.debug_tuple("InvalidId").field(id).finish(),
            Self::InvalidPrivateKey => write!(f, "InvalidPrivateKey"),
            Self::GotPublicKeyExpectedPrivateKey => write!(f, "GotPublicKeyExpectedPrivateKey"),
            Self::NonFungibleGlobalIdError(err) => f
                .debug_tuple("NonFungibleGlobalIdError")
                .field(err)
                .finish(),
            Self::FailedToBuildArguments(err) => {
                f.debug_tuple("FailedToBuildArguments").field(err).finish()
            }
            Self::ParseNetworkError(err) => f.debug_tuple("ParseNetworkError").field(err).finish(),
            Self::OwnerBadgeNotSpecified => write!(f, "OwnerBadgeNotSpecified"),
            Self::InstructionSchemaValidationError(error) => f
                .debug_tuple("InstructionSchemaValidationError")
                .field(error)
                .finish(),
            Self::InvalidResourceSpecifier(s) => {
                f.debug_tuple("InvalidResourceSpecifier").field(s).finish()
            }
            Self::RemoteGenericSubstitutionNotSupported => {
                write!(f, "RemoteGenericSubstitutionNotSupported")
            }
        }
    }
}
