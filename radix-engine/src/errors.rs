use crate::blueprints::access_controller::AccessControllerError;
use crate::blueprints::account::AccountError;
use crate::blueprints::consensus_manager::{ConsensusManagerError, ValidatorError};
use crate::blueprints::package::PackageError;
use crate::blueprints::pool::multi_resource_pool::MultiResourcePoolError;
use crate::blueprints::pool::one_resource_pool::OneResourcePoolError;
use crate::blueprints::pool::two_resource_pool::TwoResourcePoolError;
use crate::blueprints::resource::{AuthZoneError, NonFungibleVaultError};
use crate::blueprints::resource::{
    BucketError, FungibleResourceManagerError, NonFungibleResourceManagerError, ProofError,
    VaultError, WorktopError,
};
use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::kernel::call_frame::{
    CallFrameRemoveSubstateError, CallFrameScanSortedSubstatesError, CallFrameScanSubstateError,
    CallFrameSetSubstateError, CallFrameTakeSortedSubstatesError, CloseSubstateError,
    CreateFrameError, CreateNodeError, DropNodeError, ListNodeModuleError, MoveModuleError,
    OpenSubstateError, PassMessageError, ReadSubstateError, WriteSubstateError,
};
use crate::system::node_modules::access_rules::AccessRulesError;
use crate::system::node_modules::metadata::MetadataPanicError;
use crate::system::node_modules::royalty::ComponentRoyaltyError;
use crate::system::system_modules::auth::AuthError;
use crate::system::system_modules::costing::CostingError;
use crate::system::system_modules::limits::TransactionLimitsError;
use crate::system::system_modules::node_move::NodeMoveError;
use crate::transaction::AbortReason;
use crate::types::*;
use crate::vm::wasm::WasmRuntimeError;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::blueprints::package::CanonicalBlueprintId;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IdAllocationError {
    OutOfID,
}

pub trait CanBeAbortion {
    fn abortion(&self) -> Option<&AbortReason>;
}

/// Represents an error which causes a transaction to be rejected.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RejectionError {
    SuccessButFeeLoanNotRepaid,
    ErrorBeforeFeeLoanRepaid(RuntimeError),
    TransactionEpochNotYetValid {
        valid_from: Epoch,
        current_epoch: Epoch,
    },
    TransactionEpochNoLongerValid {
        valid_until: Epoch,
        current_epoch: Epoch,
    },
    IntentHashPreviouslyCommitted,
    IntentHashPreviouslyCancelled,
}

impl fmt::Display for RejectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an error when executing a transaction.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RuntimeError {
    /// An error occurred within the kernel.
    KernelError(KernelError),

    /// An error occurred within the system, notably the ClientApi implementation.
    SystemError(SystemError),

    /// An error occurred within a specific system module, like auth, costing and royalty.
    /// TODO: merge into SystemError?
    SystemModuleError(SystemModuleError),

    /// An error issued by the system when invoking upstream (such as blueprints, node modules).
    /// TODO: merge into SystemError?
    SystemUpstreamError(SystemUpstreamError),

    /// An error occurred in the vm layer
    VmError(VmError),

    /// An error occurred within application logic, like the RE models.
    ApplicationError(ApplicationError),
}

impl RuntimeError {
    pub const fn update_substate(e: CloseSubstateError) -> Self {
        Self::KernelError(KernelError::CallFrameError(
            CallFrameError::CloseSubstateError(e),
        ))
    }
}

impl From<KernelError> for RuntimeError {
    fn from(error: KernelError) -> Self {
        RuntimeError::KernelError(error.into())
    }
}

impl From<SystemUpstreamError> for RuntimeError {
    fn from(error: SystemUpstreamError) -> Self {
        RuntimeError::SystemUpstreamError(error.into())
    }
}

impl From<SystemModuleError> for RuntimeError {
    fn from(error: SystemModuleError) -> Self {
        RuntimeError::SystemModuleError(error.into())
    }
}

impl From<ApplicationError> for RuntimeError {
    fn from(error: ApplicationError) -> Self {
        RuntimeError::ApplicationError(error.into())
    }
}

impl CanBeAbortion for RuntimeError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            RuntimeError::KernelError(_) => None,
            RuntimeError::VmError(_) => None,
            RuntimeError::SystemError(_) => None,
            RuntimeError::SystemUpstreamError(_) => None,
            RuntimeError::SystemModuleError(err) => err.abortion(),
            RuntimeError::ApplicationError(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KernelError {
    // Call frame
    CallFrameError(CallFrameError),
    NodeOrphaned(NodeId),

    // ID allocation
    IdAllocationError(IdAllocationError),

    // Reference management
    InvalidDirectAccess,
    InvalidReference(NodeId),

    // Substate lock/read/write/unlock
    LockDoesNotExist(LockHandle),

    // Invoke
    InvalidInvokeAccess,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InvalidDropNodeAccess {
    pub node_id: NodeId,
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl CanBeAbortion for VmError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            VmError::Wasm(err) => err.abortion(),
            _ => None,
        }
    }
}

impl From<CallFrameError> for KernelError {
    fn from(value: CallFrameError) -> Self {
        KernelError::CallFrameError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameError {
    CreateFrameError(CreateFrameError),
    PassMessageError(PassMessageError),

    CreateNodeError(CreateNodeError),
    DropNodeError(DropNodeError),

    ListNodeModuleError(ListNodeModuleError),
    MoveModuleError(MoveModuleError),

    OpenSubstateError(OpenSubstateError),
    CloseSubstateError(CloseSubstateError),
    ReadSubstateError(ReadSubstateError),
    WriteSubstateError(WriteSubstateError),

    ScanSubstatesError(CallFrameScanSubstateError),
    TakeSubstatesError(CallFrameTakeSortedSubstatesError),
    ScanSortedSubstatesError(CallFrameScanSortedSubstatesError),
    SetSubstatesError(CallFrameSetSubstateError),
    RemoveSubstatesError(CallFrameRemoveSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemError {
    InvalidObjectHandle,
    NodeIdNotExist,
    GlobalAddressDoesNotExist,
    NoParent,
    NotAnAddressReservation,
    NotAnObject,
    NotAMethod,
    OuterObjectDoesNotExist,
    NotAFieldLock,
    NotAFieldWriteLock,
    FieldDoesNotExist(BlueprintId, u8),
    KeyValueStoreDoesNotExist(BlueprintId, u8),
    SortedIndexDoesNotExist(BlueprintId, u8),
    IndexDoesNotExist(BlueprintId, u8),
    MutatingImmutableSubstate,
    NotAKeyValueStore,
    CannotStoreOwnedInIterable,
    InvalidSubstateWrite(String),
    InvalidKeyValueStoreOwnership,
    InvalidKeyValueKey(String),
    NotAKeyValueWriteLock,
    InvalidLockFlags,
    InvalidKeyValueStoreSchema(SchemaValidationError),
    CannotGlobalize(CannotGlobalizeError),
    MissingModule(ObjectModuleId),
    InvalidModuleSet(Box<InvalidModuleSet>),
    InvalidGlobalAddressReservation,
    InvalidChildObjectCreation,
    InvalidModuleType(Box<InvalidModuleType>),
    CreateObjectError(Box<CreateObjectError>),
    InvalidInstanceSchema,
    InvalidFeature(String),
    AssertAccessRuleFailed,
    BlueprintDoesNotExist(CanonicalBlueprintId),
    AuthTemplateDoesNotExist(CanonicalBlueprintId),
    InvalidDropNodeAccess(Box<InvalidDropNodeAccess>),
    InvalidScryptoValue(DecodeError),
    CostingModuleNotEnabled,
    AuthModuleNotEnabled,
    TransactionRuntimeModuleNotEnabled,
    PayloadValidationAgainstSchemaError(PayloadValidationAgainstSchemaError),
    EventError(EventError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    SchemaNotFoundError {
        blueprint: BlueprintId,
        event_name: String,
    },
    EventSchemaNotMatch(String),
    NoAssociatedPackage,
    InvalidActor,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemUpstreamError {
    SystemFunctionCallNotAllowed,

    FnNotFound(String),
    ReceiverNotMatch(String),

    InputDecodeError(DecodeError),
    InputSchemaNotMatch(String, String),

    OutputDecodeError(DecodeError),
    OutputSchemaNotMatch(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VmError {
    Native(NativeRuntimeError),
    Wasm(WasmRuntimeError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NativeRuntimeError {
    InvalidCodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateObjectError {
    BlueprintNotFound(String),
    InvalidModule,
    WrongNumberOfKeyValueStores(BlueprintId, usize, usize),
    WrongNumberOfSubstates(BlueprintId, usize, usize),
    SchemaValidationError(BlueprintId, String),
    InvalidSubstateWrite(String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemModuleError {
    NodeMoveError(NodeMoveError),
    AuthError(AuthError),
    CostingError(CostingError),
    TransactionLimitsError(TransactionLimitsError),
    EventError(Box<EventError>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PayloadValidationAgainstSchemaError {
    BlueprintDoesNotExist(BlueprintId),
    CollectionDoesNotExist,
    FieldDoesNotExist(u8),
    KeyValueStoreKeyDoesNotExist,
    KeyValueStoreValueDoesNotExist,
    EventDoesNotExist(String),
    PayloadValidationError(String),
    InstanceSchemaDoesNotExist,
    SchemaNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InvalidModuleType {
    pub expected_blueprint: BlueprintId,
    pub actual_blueprint: BlueprintId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CannotGlobalizeError {
    NotAnObject,
    AlreadyGlobalized,
    InvalidBlueprintId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InvalidModuleSet(pub BTreeSet<ObjectModuleId>);

impl CanBeAbortion for SystemModuleError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::CostingError(err) => err.abortion(),
            _ => None,
        }
    }
}

impl From<NodeMoveError> for SystemModuleError {
    fn from(error: NodeMoveError) -> Self {
        Self::NodeMoveError(error)
    }
}

impl From<AuthError> for SystemModuleError {
    fn from(error: AuthError) -> Self {
        Self::AuthError(error)
    }
}

impl From<CostingError> for SystemModuleError {
    fn from(error: CostingError) -> Self {
        Self::CostingError(error)
    }
}

/// This enum is to help with designing intuitive error abstractions.
/// Each engine module can have its own [`SelfError`], but can also wrap arbitrary downstream errors.
/// Ultimately these errors get flattened out to a [`RuntimeError`] anyway.
#[derive(Debug, Clone)]
pub enum InvokeError<E: SelfError> {
    SelfError(E),
    Downstream(RuntimeError),
}

/// This is a trait for the non-Downstream part of [`InvokeError`]
/// We can't use `Into<RuntimeError>` because we need [`RuntimeError`] _not_ to implement it.
pub trait SelfError {
    fn into_runtime_error(self) -> RuntimeError;
}

impl<E: Into<ApplicationError>> SelfError for E {
    fn into_runtime_error(self) -> RuntimeError {
        self.into().into()
    }
}

impl<E: SelfError> From<RuntimeError> for InvokeError<E> {
    fn from(runtime_error: RuntimeError) -> Self {
        InvokeError::Downstream(runtime_error)
    }
}

impl<E: SelfError> From<E> for InvokeError<E> {
    fn from(error: E) -> Self {
        InvokeError::SelfError(error)
    }
}

impl<E: SelfError> InvokeError<E> {
    pub fn error(error: E) -> Self {
        InvokeError::SelfError(error)
    }

    pub fn downstream(runtime_error: RuntimeError) -> Self {
        InvokeError::Downstream(runtime_error)
    }
}

impl<E: SelfError> From<InvokeError<E>> for RuntimeError {
    fn from(error: InvokeError<E>) -> Self {
        match error {
            InvokeError::Downstream(runtime_error) => runtime_error,
            InvokeError::SelfError(e) => e.into_runtime_error(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ApplicationError {
    //===================
    // General errors
    //===================
    // TODO: this should never happen because of schema check?
    ExportDoesNotExist(String),

    // TODO: this should never happen because of schema check?
    InputDecodeError(DecodeError),

    Panic(String),

    //===================
    // Node module errors
    //===================
    AccessRulesError(AccessRulesError),

    MetadataError(MetadataPanicError),

    ComponentRoyaltyError(ComponentRoyaltyError),

    //===================
    // Blueprint errors
    //===================
    TransactionProcessorError(TransactionProcessorError),

    PackageError(PackageError),

    ConsensusManagerError(ConsensusManagerError),

    ValidatorError(ValidatorError),

    FungibleResourceManagerError(FungibleResourceManagerError),

    NonFungibleResourceManagerError(NonFungibleResourceManagerError),

    BucketError(BucketError),

    ProofError(ProofError),

    NonFungibleVaultError(NonFungibleVaultError),

    VaultError(VaultError),

    WorktopError(WorktopError),

    AuthZoneError(AuthZoneError),

    AccountError(AccountError),

    AccessControllerError(AccessControllerError),

    OneResourcePoolError(OneResourcePoolError),

    TwoResourcePoolError(TwoResourcePoolError),

    MultiResourcePoolError(MultiResourcePoolError),
}

impl From<TransactionProcessorError> for ApplicationError {
    fn from(value: TransactionProcessorError) -> Self {
        Self::TransactionProcessorError(value)
    }
}

impl From<PackageError> for ApplicationError {
    fn from(value: PackageError) -> Self {
        Self::PackageError(value)
    }
}

impl From<ConsensusManagerError> for ApplicationError {
    fn from(value: ConsensusManagerError) -> Self {
        Self::ConsensusManagerError(value)
    }
}

impl From<FungibleResourceManagerError> for ApplicationError {
    fn from(value: FungibleResourceManagerError) -> Self {
        Self::FungibleResourceManagerError(value)
    }
}

impl From<AccessRulesError> for ApplicationError {
    fn from(value: AccessRulesError) -> Self {
        Self::AccessRulesError(value)
    }
}

impl From<BucketError> for ApplicationError {
    fn from(value: BucketError) -> Self {
        Self::BucketError(value)
    }
}

impl From<ProofError> for ApplicationError {
    fn from(value: ProofError) -> Self {
        Self::ProofError(value)
    }
}

impl From<VaultError> for ApplicationError {
    fn from(value: VaultError) -> Self {
        Self::VaultError(value)
    }
}

impl From<WorktopError> for ApplicationError {
    fn from(value: WorktopError) -> Self {
        Self::WorktopError(value)
    }
}

impl From<AuthZoneError> for ApplicationError {
    fn from(value: AuthZoneError) -> Self {
        Self::AuthZoneError(value)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<OpenSubstateError> for CallFrameError {
    fn from(value: OpenSubstateError) -> Self {
        Self::OpenSubstateError(value)
    }
}

impl From<CloseSubstateError> for CallFrameError {
    fn from(value: CloseSubstateError) -> Self {
        Self::CloseSubstateError(value)
    }
}

impl From<PassMessageError> for CallFrameError {
    fn from(value: PassMessageError) -> Self {
        Self::PassMessageError(value)
    }
}
