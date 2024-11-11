use crate::blueprints::access_controller::AccessControllerError;
use crate::blueprints::account::AccountError;
use crate::blueprints::consensus_manager::{ConsensusManagerError, ValidatorError};
use crate::blueprints::package::PackageError;
use crate::blueprints::pool::v1::errors::{
    multi_resource_pool::Error as MultiResourcePoolError,
    one_resource_pool::Error as OneResourcePoolError,
    two_resource_pool::Error as TwoResourcePoolError,
};
use crate::blueprints::resource::{AuthZoneError, NonFungibleVaultError};
use crate::blueprints::resource::{
    BucketError, FungibleResourceManagerError, NonFungibleResourceManagerError, ProofError,
    VaultError, WorktopError,
};
use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::internal_prelude::*;
use crate::kernel::call_frame::{
    CallFrameDrainSubstatesError, CallFrameRemoveSubstateError, CallFrameScanKeysError,
    CallFrameScanSortedSubstatesError, CallFrameSetSubstateError, CloseSubstateError,
    CreateFrameError, CreateNodeError, DropNodeError, MarkTransientSubstateError,
    MovePartitionError, OpenSubstateError, PassMessageError, PinNodeError, ReadSubstateError,
    WriteSubstateError,
};
use crate::object_modules::metadata::MetadataError;
use crate::object_modules::role_assignment::RoleAssignmentError;
use crate::object_modules::royalty::ComponentRoyaltyError;
use crate::system::system_modules::auth::AuthError;
use crate::system::system_modules::costing::CostingError;
use crate::system::system_modules::limits::TransactionLimitsError;
use crate::system::system_type_checker::TypeCheckError;
use crate::transaction::AbortReason;
use crate::vm::wasm::WasmRuntimeError;
use crate::vm::ScryptoVmVersionError;
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::{ActorStateHandle, AttachedModuleId};
use radix_engine_interface::blueprints::package::{BlueprintPartitionType, CanonicalBlueprintId};
use radix_transactions::model::IntentHash;
use sbor::representations::PrintMode;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IdAllocationError {
    OutOfID,
}

pub trait CanBeAbortion {
    fn abortion(&self) -> Option<&AbortReason>;
}

pub mod error_models {
    use radix_common::prelude::*;

    /// This is a special NodeId which gets encoded as a reference in SBOR...
    /// This means that it can be rendered as a string in the output.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
    #[sbor(
        as_type = "Reference",
        as_ref = "&Reference(self.0)",
        from_value = "Self(value.0)",
        type_name = "NodeId"
    )]
    pub struct ReferencedNodeId(pub radix_common::prelude::NodeId);

    impl Debug for ReferencedNodeId {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl From<radix_common::prelude::NodeId> for ReferencedNodeId {
        fn from(value: radix_common::prelude::NodeId) -> Self {
            Self(value)
        }
    }

    /// This is a special NodeId which gets encoded as a reference in SBOR...
    /// This means that it can be rendered as a string in the output.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
    #[sbor(
        as_type = "Own",
        as_ref = "&Own(self.0)",
        from_value = "Self(value.0)",
        type_name = "NodeId"
    )]
    pub struct OwnedNodeId(pub radix_common::prelude::NodeId);

    impl Debug for OwnedNodeId {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl From<radix_common::prelude::NodeId> for OwnedNodeId {
        fn from(value: radix_common::prelude::NodeId) -> Self {
            Self(value)
        }
    }
}

lazy_static::lazy_static! {
    /// See [`HISTORIC_RUNTIME_ERROR_SCHEMAS`] for more information.
    ///
    /// Although the RejectionReason isn't used on the node, we do a similar thing anyway.
    static ref HISTORIC_REJECTION_REASON_SCHEMAS: [ScryptoSingleTypeSchema; 1] = {
        [
            ScryptoSingleTypeSchema::from(include_bytes!("rejection_reason_cuttlefish_schema.bin")),
        ]
    };
}

/// Represents an error which causes a transaction to be rejected.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
// #[derive(ScryptoSborAssertion)]
// #[sbor_assert(fixed("FILE:rejection_reason_[NEW-VERSION-NAME]_schema.bin"), generate)]
// #[sbor_assert(fixed("FILE:rejection_reason_[UNDEPLOYED-CURRENT-VERSION-NAME]_schema.bin"), regenerate)]
pub enum RejectionReason {
    TransactionEpochNotYetValid {
        /// `start_epoch_inclusive`
        valid_from: Epoch,
        current_epoch: Epoch,
    },
    TransactionEpochNoLongerValid {
        /// One epoch before `end_epoch_exclusive`
        valid_until: Epoch,
        current_epoch: Epoch,
    },
    TransactionProposerTimestampNotYetValid {
        valid_from_inclusive: Instant,
        current_time: Instant,
    },
    TransactionProposerTimestampNoLongerValid {
        valid_to_exclusive: Instant,
        current_time: Instant,
    },
    IntentHashPreviouslyCommitted(IntentHash),
    IntentHashPreviouslyCancelled(IntentHash),

    BootloadingError(BootloadingError),

    ErrorBeforeLoanAndDeferredCostsRepaid(RuntimeError),
    SuccessButFeeLoanNotRepaid,
    SubintentsNotYetSupported,
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for RejectionReason {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext,
    ) -> Result<(), Self::Error> {
        self.create_persistable().contextual_format(f, context)
    }
}

impl RejectionReason {
    pub fn create_persistable(&self) -> PersistableRejectionReason {
        PersistableRejectionReason {
            schema_index: HISTORIC_REJECTION_REASON_SCHEMAS.len() as u32 - 1,
            encoded_rejection_reason: scrypto_decode(&scrypto_encode(self).unwrap()).unwrap(),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct PersistableRejectionReason {
    pub schema_index: u32,
    pub encoded_rejection_reason: ScryptoOwnedRawValue,
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for PersistableRejectionReason {
    type Error = fmt::Error;

    /// See [`SerializableRuntimeError::contextual_format`] for more information.
    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext,
    ) -> Result<(), Self::Error> {
        let value = &self.encoded_rejection_reason;
        let formatted_optional = HISTORIC_REJECTION_REASON_SCHEMAS
            .get(self.schema_index as usize)
            .and_then(|schema| {
                format_debug_like_value(
                    f,
                    schema,
                    value,
                    sbor::representations::PrintMode::SingleLine,
                    *context,
                )
            });
        match formatted_optional {
            Some(result) => result,
            None => match scrypto_encode(&value) {
                Ok(encoded) => write!(f, "UnknownRejectionReason({})", hex::encode(encoded)),
                Err(error) => write!(f, "CannotDisplayRejectionReason({error:?})"),
            },
        }
    }
}

fn format_debug_like_value(
    f: &mut impl fmt::Write,
    schema: &SingleTypeSchema<ScryptoCustomSchema>,
    value: &ScryptoRawValue,
    print_mode: PrintMode,
    custom_context: ScryptoValueDisplayContext,
) -> Option<fmt::Result> {
    use sbor::representations::*;
    let type_id = schema.type_id;
    let schema = schema.schema.as_unique_version();
    let depth_limit = SCRYPTO_SBOR_V1_MAX_DEPTH;

    // Sanity check this is the correct schema...
    validate_partial_payload_against_schema::<ScryptoCustomExtension, _>(
        value.value_body_bytes(),
        traversal::ExpectedStart::ValueBody(value.value_kind()),
        true,
        0,
        schema,
        type_id,
        &(),
        depth_limit,
    )
    .ok()?;

    // Then encode it...
    let display_parameters = ValueDisplayParameters::Annotated {
        display_mode: DisplayMode::RustLike(RustLikeOptions::debug_like()),
        print_mode,
        custom_context,
        schema,
        type_id,
        depth_limit,
    };

    Some(write!(f, "{}", value.display(display_parameters)))
}

impl From<BootloadingError> for RejectionReason {
    fn from(value: BootloadingError) -> Self {
        RejectionReason::BootloadingError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionExecutionError {
    /// An error ocurred when bootloading a kernel.
    BootloadingError(BootloadingError),

    /// A runtime error
    RuntimeError(RuntimeError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BootloadingError {
    ReferencedNodeDoesNotExist(error_models::ReferencedNodeId),
    ReferencedNodeIsNotAnObject(error_models::ReferencedNodeId),
    ReferencedNodeDoesNotAllowDirectAccess(error_models::ReferencedNodeId),

    FailedToApplyDeferredCosts(CostingError),
}

lazy_static::lazy_static! {
    /// This list is used to render string messages from historically stored
    /// [`SerializableRuntimeError`]s in the LocalTransactionExecution index in the node.
    ///
    /// In particular, each [`SerializableRuntimeError`] stores the index of the current schema
    /// in this array at the time it was created.
    ///
    /// But of course, when the error is read (e.g. in the Core API stream 10 months later),
    /// we may be a few protocol versions down the line, and the `RuntimeError` schema may have changed.
    ///
    /// To get around this, we simply use the stored schema index to look up the correct schema here.
    /// And we use this historic schema to render the error message.
    ///
    /// This allows us the following benefits:
    /// * We can use a condensed error encoding (rather than just storing it as a string)
    /// * We can change the `RuntimeError` schema freely, as long as we ensure old schemas are kept here.
    ///   Tests will ensure we don't break this.
    ///
    /// We MUST NOT change/remove/reorder existing schemas in this list, if they have been released
    /// in a node version. This is to ensure that we can always decode old errors.
    ///
    /// New schemas can be generated with `#[sbor_assert(fixed("FILE:xxx"))]` generator above.
    static ref HISTORIC_RUNTIME_ERROR_SCHEMAS: [ScryptoSingleTypeSchema; 2] = {
        [
            ScryptoSingleTypeSchema::from(include_bytes!("runtime_error_pre_cuttlefish_schema.bin")),
            ScryptoSingleTypeSchema::from(include_bytes!("runtime_error_cuttlefish_schema.bin")),
        ]
    };
}

/// Represents an error when executing a transaction.
#[derive(Clone, PartialEq, Eq, ScryptoSbor, Debug)]
// You are welcome to update the RuntimeError structure, but the tests will make you ensure
// that the current schema is the last in HISTORIC_RUNTIME_ERROR_SCHEMAS above, and that
// any schema used in a released node version never gets removed.
//
// What this means is:
// - You may regenerate the schema for the current version, if it's never been released.
// - Otherwise, you will want to generate a new schema for the new version.
//
// So:
// - Temporarily uncomment the derive, and one of the sbor_assert lines below
// - Rename the file in the line
// - Run the test to (re)generate the schema
// - Revert the changes to these few lines
// - If it's a new schema, add it to the HISTORIC_RUNTIME_ERROR_SCHEMAS list above.
// - Check the `the_current_runtime_schema_is_last_on_historic_runtime_list` test passes.
//
// #[derive(ScryptoSborAssertion)]
// #[sbor_assert(fixed("FILE:runtime_error_[NEW-VERSION-NAME]_schema.bin"), generate)]
// #[sbor_assert(fixed("FILE:runtime_error_[UNDEPLOYED-CURRENT-VERSION-NAME]_schema.bin"), regenerate)]
pub enum RuntimeError {
    /// An error occurred within the kernel.
    KernelError(KernelError),

    /// An error occurred within the system, notably the SystemAPI implementation.
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

    FinalizationCostingError(CostingError),
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for RuntimeError {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext,
    ) -> Result<(), Self::Error> {
        self.create_persistable().contextual_format(f, context)
    }
}

impl RuntimeError {
    pub fn create_persistable(&self) -> PersistableRuntimeError {
        PersistableRuntimeError {
            schema_index: HISTORIC_RUNTIME_ERROR_SCHEMAS.len() as u32 - 1,
            encoded_error: scrypto_decode(&scrypto_encode(self).unwrap()).unwrap(),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct PersistableRuntimeError {
    pub schema_index: u32,
    // RawValue and RawPayload will change in https://github.com/radixdlt/radixdlt-scrypto/pull/1860
    // It's important we stick with `RawValue` here (so it encodes/decode as SBOR itself),
    // but ideally it would be a full payload underneath. This can be the case from #1860.
    pub encoded_error: ScryptoOwnedRawValue,
}

/// This is used to render the error message, with a fallback if an invalid schema
/// is associated with the error.
///
/// This fallback is necessary due to historic breakages of backwards compatibility
/// in the `RuntimeError` type structure.
///
/// Specifically, at anemone / bottlenose, there were very minor changes, which affected
/// a tiny minority of errors. If we could find the historic schemas, we could actually
/// render them properly here. Unfortunately, the historic schemas are not easy to find out
/// (it would require backporting the schema generation logic), so instead we just have
/// a fallback for these cases.
///
/// This fallback will only be applied on nodes, when returning occasional errors for
/// old transactions that haven't resynced since Bottlenose.
impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for PersistableRuntimeError {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext,
    ) -> Result<(), Self::Error> {
        let value = &self.encoded_error;
        let formatted_optional = HISTORIC_RUNTIME_ERROR_SCHEMAS
            .get(self.schema_index as usize)
            .and_then(|schema| {
                format_debug_like_value(f, schema, value, PrintMode::SingleLine, *context)
            });
        match formatted_optional {
            Some(result) => result,
            None => match scrypto_encode(&value) {
                Ok(encoded) => write!(f, "UnknownError({})", hex::encode(encoded)),
                Err(error) => write!(f, "CannotDisplayError({error:?})"),
            },
        }
    }
}

impl SystemApiError for RuntimeError {}

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
            RuntimeError::FinalizationCostingError(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KernelError {
    // Call frame
    CallFrameError(CallFrameError),

    // ID allocation
    IdAllocationError(IdAllocationError),

    // Substate lock/read/write/unlock
    SubstateHandleDoesNotExist(SubstateHandle),

    OrphanedNodes(Vec<error_models::OwnedNodeId>),

    StackError(StackError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InvalidDropAccess {
    pub node_id: error_models::ReferencedNodeId,
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub actor_package: Option<PackageAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InvalidGlobalizeAccess {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub actor_package: Option<PackageAddress>,
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
    PinNodeError(PinNodeError),

    MovePartitionError(MovePartitionError),

    MarkTransientSubstateError(MarkTransientSubstateError),
    OpenSubstateError(OpenSubstateError),
    CloseSubstateError(CloseSubstateError),
    ReadSubstateError(ReadSubstateError),
    WriteSubstateError(WriteSubstateError),

    ScanSubstatesError(CallFrameScanKeysError),
    DrainSubstatesError(CallFrameDrainSubstatesError),
    ScanSortedSubstatesError(CallFrameScanSortedSubstatesError),
    SetSubstatesError(CallFrameSetSubstateError),
    RemoveSubstatesError(CallFrameRemoveSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum StackError {
    InvalidStackId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemError {
    NoBlueprintId,
    NoPackageAddress,
    InvalidActorStateHandle,
    InvalidActorRefHandle,

    GlobalizingTransientBlueprint,
    GlobalAddressDoesNotExist,
    NotAnAddressReservation,
    NotAnObject,
    NotAKeyValueStore,
    ModulesDontHaveOuterObjects,
    ActorNodeIdDoesNotExist,
    OuterObjectDoesNotExist,
    NotAFieldHandle,
    NotAFieldWriteHandle,
    RootHasNoType,
    AddressBech32EncodeError,
    TypeCheckError(TypeCheckError),
    FieldDoesNotExist(BlueprintId, u8),
    CollectionIndexDoesNotExist(BlueprintId, u8),
    CollectionIndexIsOfWrongType(
        BlueprintId,
        u8,
        BlueprintPartitionType,
        BlueprintPartitionType,
    ),
    KeyValueEntryLocked,
    FieldLocked(ActorStateHandle, u8),
    ObjectModuleDoesNotExist(AttachedModuleId),
    NotAKeyValueEntryHandle,
    NotAKeyValueEntryWriteHandle,
    InvalidLockFlags,
    CannotGlobalize(CannotGlobalizeError),
    MissingModule(ModuleId),
    InvalidGlobalAddressReservation,
    InvalidChildObjectCreation,
    InvalidModuleType(Box<InvalidModuleType>),
    CreateObjectError(Box<CreateObjectError>),
    InvalidGenericArgs,
    InvalidFeature(String),
    AssertAccessRuleFailed,
    BlueprintDoesNotExist(CanonicalBlueprintId),
    AuthTemplateDoesNotExist(CanonicalBlueprintId),
    InvalidGlobalizeAccess(Box<InvalidGlobalizeAccess>),
    InvalidDropAccess(Box<InvalidDropAccess>),
    CostingModuleNotEnabled,
    AuthModuleNotEnabled,
    TransactionRuntimeModuleNotEnabled,
    ForceWriteEventFlagsNotAllowed,

    BlueprintTypeNotFound(String),

    BlsError(String),
    InputDataEmpty,

    /// A panic that's occurred in the system-layer or below. We're calling it system panic since
    /// we're treating the system as a black-box here.
    ///
    /// Note that this is only used when feature std is used.
    SystemPanic(String),

    CannotLockFeeInChildSubintent(usize),
    IntentError(IntentError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IntentError {
    CannotVerifyParentOnRoot,
    CannotYieldProof,
    VerifyParentFailed,
    InvalidIntentIndex(usize),
    NoParentToYieldTo,
    AssertNextCallReturnsFailed(ResourceConstraintsError),
    AssertBucketContentsFailed(ResourceConstraintError),
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
    HookNotFound(BlueprintHook),

    InputDecodeError(DecodeError),
    InputSchemaNotMatch(String, String),

    OutputDecodeError(DecodeError),
    OutputSchemaNotMatch(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VmError {
    Native(NativeRuntimeError),
    Wasm(WasmRuntimeError),
    ScryptoVmVersion(ScryptoVmVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NativeRuntimeError {
    InvalidCodeId,

    /// A panic was encountered in Native code.
    Trap {
        export_name: String,
        input: ScryptoValue,
        error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateObjectError {
    BlueprintNotFound(String),
    InvalidFieldDueToFeature(BlueprintId, u8),
    MissingField(BlueprintId, u8),
    InvalidFieldIndex(BlueprintId, u8),
    SchemaValidationError(BlueprintId, String),
    InvalidSubstateWrite(String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemModuleError {
    AuthError(AuthError),
    CostingError(CostingError),
    TransactionLimitsError(TransactionLimitsError),
    EventError(Box<EventError>),
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

impl CanBeAbortion for SystemModuleError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::CostingError(err) => err.abortion(),
            _ => None,
        }
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

    /// A panic.
    PanicMessage(String),

    //===================
    // Node module errors
    //===================
    RoleAssignmentError(RoleAssignmentError),

    MetadataError(MetadataError),

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

impl From<RoleAssignmentError> for ApplicationError {
    fn from(value: RoleAssignmentError) -> Self {
        Self::RoleAssignmentError(value)
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

impl From<MovePartitionError> for CallFrameError {
    fn from(value: MovePartitionError) -> Self {
        Self::MovePartitionError(value)
    }
}

impl From<ReadSubstateError> for CallFrameError {
    fn from(value: ReadSubstateError) -> Self {
        Self::ReadSubstateError(value)
    }
}

impl From<WriteSubstateError> for CallFrameError {
    fn from(value: WriteSubstateError) -> Self {
        Self::WriteSubstateError(value)
    }
}

impl From<CreateNodeError> for CallFrameError {
    fn from(value: CreateNodeError) -> Self {
        Self::CreateNodeError(value)
    }
}

impl From<DropNodeError> for CallFrameError {
    fn from(value: DropNodeError) -> Self {
        Self::DropNodeError(value)
    }
}

impl From<CreateFrameError> for CallFrameError {
    fn from(value: CreateFrameError) -> Self {
        Self::CreateFrameError(value)
    }
}

impl From<CallFrameScanKeysError> for CallFrameError {
    fn from(value: CallFrameScanKeysError) -> Self {
        Self::ScanSubstatesError(value)
    }
}

impl From<CallFrameScanSortedSubstatesError> for CallFrameError {
    fn from(value: CallFrameScanSortedSubstatesError) -> Self {
        Self::ScanSortedSubstatesError(value)
    }
}

impl From<CallFrameDrainSubstatesError> for CallFrameError {
    fn from(value: CallFrameDrainSubstatesError) -> Self {
        Self::DrainSubstatesError(value)
    }
}

impl From<CallFrameSetSubstateError> for CallFrameError {
    fn from(value: CallFrameSetSubstateError) -> Self {
        Self::SetSubstatesError(value)
    }
}

impl From<CallFrameRemoveSubstateError> for CallFrameError {
    fn from(value: CallFrameRemoveSubstateError) -> Self {
        Self::RemoveSubstatesError(value)
    }
}

impl<T> From<T> for RuntimeError
where
    T: Into<CallFrameError>,
{
    fn from(value: T) -> Self {
        Self::KernelError(KernelError::CallFrameError(value.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_runtime_error_schema_is_last_on_historic_list() {
        let latest = HISTORIC_RUNTIME_ERROR_SCHEMAS.last().unwrap();
        let current = generate_single_type_schema::<RuntimeError, ScryptoCustomSchema>();

        // If this test fails, see the comment above `RuntimeError` for instructions.
        compare_single_type_schemas(
            &SchemaComparisonSettings::require_equality(),
            latest,
            &current,
        )
        .assert_valid("latest", "current");
    }

    #[test]
    fn the_current_runtime_error_schema_has_no_raw_node_ids() {
        let current = generate_single_type_schema::<RuntimeError, ScryptoCustomSchema>();
        assert_no_raw_node_ids(&current);
    }

    #[test]
    fn the_current_rejection_reason_schema_is_last_on_historic_list() {
        let latest = HISTORIC_REJECTION_REASON_SCHEMAS.last().unwrap();
        let current = generate_single_type_schema::<RejectionReason, ScryptoCustomSchema>();

        // If this test fails, see the comment above `RejectionReason` for instructions.
        compare_single_type_schemas(
            &SchemaComparisonSettings::require_equality(),
            latest,
            &current,
        )
        .assert_valid("latest", "current");
    }

    #[test]
    fn the_current_rejection_reason_schema_has_no_raw_node_ids() {
        let current = generate_single_type_schema::<RejectionReason, ScryptoCustomSchema>();
        assert_no_raw_node_ids(&current);
    }

    fn assert_no_raw_node_ids(schema: &SingleTypeSchema<ScryptoCustomSchema>) {
        let schema = schema.schema.as_unique_version();
        for (type_kind, type_metadata) in schema.type_kinds.iter().zip(schema.type_metadata.iter())
        {
            if type_metadata.type_name.as_deref() == Some("NodeId") {
                match type_kind {
                    TypeKind::Custom(ScryptoCustomTypeKind::Own)
                    | TypeKind::Custom(ScryptoCustomTypeKind::Reference) => {}
                    _ => {
                        let mut formatted_schema = String::new();
                        format_debug_like_value(
                            &mut formatted_schema,
                            &generate_single_type_schema::<
                                SchemaV1<ScryptoCustomSchema>,
                                ScryptoCustomSchema,
                            >(),
                            &scrypto_decode(&scrypto_encode(schema).unwrap()).unwrap(),
                            PrintMode::MultiLine {
                                indent_size: 4,
                                base_indent: 4,
                                first_line_indent: 4,
                            },
                            ScryptoValueDisplayContext::default(),
                        );
                        // If this is too much for the console, use:
                        // cargo test --package radix-engine --lib -- errors::tests::the_current_rejection_reason_schema_has_no_raw_node_ids --exact --show-output > output.txt
                        // And then check for some of the type definitions of the type names preceeding the last mention of "NodeId"
                        // One of these types will directly mention `NodeId` instead of `error_models::ReferencedNodeId` or `error_models::OwnedNodeId`.
                        panic!("A raw NodeId was detected somewhere in the error schema. Use `error_models::ReferencedNodeId` or `error_models::OwnedNodeId` instead.\n\nSchema:\n{}", formatted_schema);
                    }
                }
            }
        }
    }

    #[test]
    fn runtime_error_string() {
        let network = NetworkDefinition::mainnet();
        let address_encoder = AddressBech32Encoder::new(&network);
        let address_encoder = Some(&address_encoder);

        // Example one - Account withdraw/lock fee/create proof error
        {
            let runtime_error = RuntimeError::ApplicationError(ApplicationError::AccountError(
                AccountError::VaultDoesNotExist {
                    resource_address: XRD,
                },
            ));

            // Old error
            let debugged = format!("{:?}", runtime_error);
            assert_eq!(debugged, "ApplicationError(AccountError(VaultDoesNotExist { resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6) }))");

            // New error
            let rendered = runtime_error.to_string(address_encoder);
            assert_eq!(rendered, "ApplicationError(AccountError(VaultDoesNotExist { resource_address: ResourceAddress(\"resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd\") }))");
        }

        // Example two - dangling bucket error
        {
            let mut id_allocator = crate::kernel::id_allocator::IdAllocator::new(hash("seed-data"));

            // Unfortunately buckets didn't get their own entity type...
            let bucket_entity_type = EntityType::InternalGenericComponent;
            let example_bucket_1 = id_allocator.allocate_node_id(bucket_entity_type).unwrap();
            let example_bucket_2 = id_allocator.allocate_node_id(bucket_entity_type).unwrap();
            let runtime_error = RuntimeError::KernelError(KernelError::OrphanedNodes(vec![
                example_bucket_1.into(),
                example_bucket_2.into(),
            ]));

            // Old error
            let debugged = format!("{:?}", runtime_error);
            assert_eq!(debugged, "KernelError(OrphanedNodes([NodeId(\"f82ee60dbc11caa1594fccdbb8031c41af8084344bcbe7a4c784491a7d4c\"), NodeId(\"f8abce267317b7bdd859951840ccd25f1ea7e83c538d507e0f82da7b9aed\")]))");

            // New error
            let rendered = runtime_error.to_string(address_encoder);
            assert_eq!(rendered, "KernelError(OrphanedNodes([NodeId(\"internal_component_rdx1lqhwvrduz892zk20endmsqcugxhcppp5f0970fx8s3y35l2vv5mzfn\"), NodeId(\"internal_component_rdx1lz4uufnnz7mmmkzej5vypnxjtu0206pu2wx4qls0std8hxhd3v84yv\")]))");
        }
    }
}
