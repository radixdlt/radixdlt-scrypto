use sbor::rust::boxed::Box;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{DecodeError, Value};
use scrypto::engine::types::*;
use transaction::errors::*;

use crate::fee::CostUnitCounterError;
use crate::model::*;
use crate::wasm::InvokeError;

/// Represents an error when executing a transaction.
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    /// Error when invoking a blueprint or component (recursive).
    InvokeError(Box<InvokeError>),

    /// The data is not a valid SBOR value.
    DecodeError(DecodeError),

    AuthZoneDoesNotExist,

    WorktopDoesNotExist,

    /// Failed to allocate an ID.
    IdAllocationError(IdAllocationError),

    /// Invalid request code.
    MethodDoesNotExist(String),
    InvalidFnInput {
        fn_ident: String,
        input: Value,
    },
    InvalidFnOutput {
        fn_ident: String,
        output: Value,
    },

    /// Package does not exist.
    PackageNotFound(PackageAddress),
    InvalidPackage(DecodeError),

    PackageError(PackageError),

    SystemError(SystemError),

    /// Blueprint does not exist.
    BlueprintNotFound(PackageAddress, String),

    ComponentReentrancy(ComponentAddress),

    /// Component does not exist.
    ComponentNotFound(ComponentAddress),

    BlueprintFunctionDoesNotExist(String),
    ComponentDecodeError(DecodeError),

    /// Resource manager does not exist.
    ResourceManagerNotFound(ResourceAddress),

    ValueNotFound(StoredValueId),

    /// Key Value Store does not exist.
    KeyValueStoreNotFound(KeyValueStoreId),

    /// Cyclic Key Value Store added
    CyclicKeyValueStore(KeyValueStoreId),

    StoredValueRemoved(StoredValueId),

    /// Bucket does not exist.
    BucketNotFound(BucketId),

    /// Proof does not exist.
    ProofNotFound(ProofId),

    /// Resource manager access error.
    ResourceManagerError(ResourceManagerError),

    /// Bucket access error.
    BucketError(BucketError),

    /// Vault access error.
    VaultError(VaultError),

    /// Worktop access error.
    WorktopError(WorktopError),

    /// Error when generating or accessing proof.
    ProofError(ProofError),

    /// Bucket is not allowed.
    BucketNotAllowed,

    /// Proof is not allowed.
    ProofNotAllowed,

    /// Vault is not allowed
    VaultNotAllowed,

    /// Key Value store is not allowed
    KeyValueStoreNotAllowed,

    /// Resource check failure.
    ResourceCheckFailure(ResourceFailure),

    /// AuthZone error
    AuthZoneError(AuthZoneError),

    /// System Authorization Failure
    AuthorizationError {
        function: String,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },

    /// Can't move a locked bucket.
    CantMoveLockedBucket,

    /// Can't move restricted proof.
    CantMoveRestrictedProof(ProofId),

    InvalidInvocation,

    CostingError(CostUnitCounterError),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ResourceFailure {
    Resource(ResourceAddress),
    Resources(Vec<ResourceAddress>),
    UnclaimedKeyValueStore,
    Unknown,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
