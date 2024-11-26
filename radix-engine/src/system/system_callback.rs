use super::module::*;
use super::system_modules::costing::{CostingModuleConfig, ExecutionCostingEntry};
use super::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::blueprints::account::ACCOUNT_CREATE_PREALLOCATED_ED25519_ID;
use crate::blueprints::account::ACCOUNT_CREATE_PREALLOCATED_SECP256K1_ID;
use crate::blueprints::consensus_manager::*;
use crate::blueprints::identity::IDENTITY_CREATE_PREALLOCATED_ED25519_ID;
use crate::blueprints::identity::IDENTITY_CREATE_PREALLOCATED_SECP256K1_ID;
use crate::blueprints::resource::fungible_vault::{DepositEvent, PayFeeEvent};
use crate::blueprints::resource::*;
use crate::blueprints::transaction_tracker::*;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::{CallFrameInit, CallFrameMessage, StableReferenceType};
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::actor::Actor;
use crate::system::actor::BlueprintHookActor;
use crate::system::actor::FunctionActor;
use crate::system::actor::MethodActor;
use crate::system::module::InitSystemModule;
use crate::system::system::SystemService;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::system::system_modules::auth::AuthModule;
use crate::system::system_modules::costing::*;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::limits::LimitsModule;
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_modules::{EnabledModules, SystemModuleMixer};
use crate::system::system_substates::KeyValueEntrySubstate;
use crate::system::system_type_checker::{BlueprintTypeTarget, KVStoreTypeTarget};
use crate::system::transaction::multithread_intent_processor::MultiThreadIntentProcessor;
use crate::track::*;
use crate::transaction::*;
use radix_blueprint_schema_init::RefTypes;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::SystemObjectApi;
use radix_engine_interface::api::{CollectionIndex, SystemBlueprintApi};
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::hooks::*;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct SystemParameters {
    pub network_definition: NetworkDefinition,
    pub costing_module_config: CostingModuleConfig,
    pub costing_parameters: CostingParameters,
    pub limit_parameters: LimitParameters,
}

impl SystemParameters {
    pub fn latest(network_definition: NetworkDefinition) -> Self {
        Self::bottlenose(network_definition)
    }

    pub fn bottlenose(network_definition: NetworkDefinition) -> Self {
        Self {
            network_definition,
            costing_module_config: CostingModuleConfig::bottlenose(),
            costing_parameters: CostingParameters::babylon_genesis(),
            limit_parameters: LimitParameters::babylon_genesis(),
        }
    }

    pub fn babylon_genesis(network_definition: NetworkDefinition) -> Self {
        Self {
            network_definition,
            costing_module_config: CostingModuleConfig::babylon_genesis(),
            costing_parameters: CostingParameters::babylon_genesis(),
            limit_parameters: LimitParameters::babylon_genesis(),
        }
    }
}

pub type SystemBootSubstate = SystemBoot;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ScryptoSborAssertion)]
#[sbor_assert(backwards_compatible(
    cuttlefish = "FILE:system_boot_substate_cuttlefish_schema.bin",
))]
pub enum SystemBoot {
    V1(SystemParameters),
    V2(SystemVersion, SystemParameters),
}

impl SystemBoot {
    /// Loads system boot from the database, or resolves a fallback..
    ///
    /// # Panics
    /// This method panics if the database is pre-bottlenose and the execution config
    /// does not specify a network definition.
    pub fn load(substate_db: &impl SubstateDatabase, execution_config: &ExecutionConfig) -> Self {
        substate_db
            .get_substate(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::SystemBoot,
            )
            .unwrap_or_else(|| {
                let overrides = execution_config.system_overrides.as_ref();
                let network_definition = overrides.and_then(|o| o.network_definition.as_ref())
                    .expect("Before bottlenose, no SystemBoot substate exists, so a network_definition must be provided in the SystemOverrides of the ExecutionConfig.");
                SystemBoot::babylon_genesis(network_definition.clone())
            })
    }

    pub fn latest(network_definition: NetworkDefinition) -> Self {
        Self::cuttlefish(network_definition)
    }

    pub fn cuttlefish(network_definition: NetworkDefinition) -> Self {
        SystemBoot::V2(
            SystemVersion::V3,
            SystemParameters::bottlenose(network_definition),
        )
    }

    pub fn cuttlefish_part1_for_previous_parameters(parameters: SystemParameters) -> Self {
        SystemBoot::V2(SystemVersion::V2, parameters)
    }

    pub fn cuttlefish_part2_for_previous_parameters(parameters: SystemParameters) -> Self {
        SystemBoot::V2(SystemVersion::V3, parameters)
    }

    pub fn bottlenose(network_definition: NetworkDefinition) -> Self {
        SystemBoot::V1(SystemParameters::bottlenose(network_definition))
    }

    pub fn babylon_genesis(network_definition: NetworkDefinition) -> Self {
        SystemBoot::V1(SystemParameters::babylon_genesis(network_definition))
    }

    pub fn system_version(&self) -> SystemVersion {
        match self {
            Self::V1(..) => SystemVersion::V1,
            Self::V2(version, _) => *version,
        }
    }

    pub fn into_parameters(self) -> SystemParameters {
        match self {
            Self::V1(parameters) => parameters,
            Self::V2(_, parameters) => parameters,
        }
    }
}

/// System logic which may change given a protocol version
#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemVersion {
    V1,
    V2,
    V3,
}

impl SystemVersion {
    pub const fn latest() -> Self {
        Self::V3
    }

    fn create_auth_module(
        &self,
        executable: &ExecutableTransaction,
    ) -> Result<AuthModule, RejectionReason> {
        let auth_module = match self {
            SystemVersion::V1 => {
                // This isn't exactly a necessary check as node logic should protect against this
                // but keep it here for sanity
                if executable.subintents().len() > 0 {
                    return Err(RejectionReason::SubintentsNotYetSupported);
                }
                let intent = executable.transaction_intent();
                AuthModule::new_with_transaction_processor_auth_zone(intent.auth_zone_init.clone())
            }
            SystemVersion::V2 | SystemVersion::V3 => AuthModule::new(),
        };

        Ok(auth_module)
    }

    fn execute_transaction<Y: SystemBasedKernelApi>(
        &self,
        api: &mut Y,
        executable: &ExecutableTransaction,
        global_address_reservations: Vec<GlobalAddressReservation>,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let output = match self {
            SystemVersion::V1 => {
                let mut system_service = SystemService::new(api);
                let intent = executable.transaction_intent();
                let rtn = system_service.call_function(
                    TRANSACTION_PROCESSOR_PACKAGE,
                    TRANSACTION_PROCESSOR_BLUEPRINT,
                    TRANSACTION_PROCESSOR_RUN_IDENT,
                    scrypto_encode(&TransactionProcessorRunInputEfficientEncodable {
                        manifest_encoded_instructions: intent.encoded_instructions.as_ref(),
                        global_address_reservations: global_address_reservations.as_slice(),
                        references: &intent.references,
                        blobs: &intent.blobs,
                    })
                    .unwrap(),
                )?;
                let output: Vec<InstructionOutput> = scrypto_decode(&rtn).unwrap();
                output
            }
            SystemVersion::V2 | SystemVersion::V3 => {
                let mut txn_threads = MultiThreadIntentProcessor::init(
                    executable,
                    global_address_reservations.as_slice(),
                    api,
                )?;
                txn_threads.execute(api)?;
                let output = txn_threads
                    .threads
                    .get_mut(0)
                    .unwrap()
                    .0
                    .outputs
                    .drain(..)
                    .collect();
                output
            }
        };

        Ok(output)
    }

    pub fn should_consume_cost_units<Y: SystemBasedKernelApi>(&self, api: &mut Y) -> bool {
        match self {
            SystemVersion::V1 => {
                // Skip client-side costing requested by TransactionProcessor
                if api.kernel_get_current_stack_depth_uncosted() == 1 {
                    return false;
                }
            }
            SystemVersion::V2 | SystemVersion::V3 => {}
        }

        true
    }

    pub fn should_inject_transaction_processor_proofs_in_call_function(&self) -> bool {
        match self {
            SystemVersion::V1 => true,
            SystemVersion::V2 | SystemVersion::V3 => false,
        }
    }

    pub fn should_charge_for_transaction_intent(&self) -> bool {
        match self {
            SystemVersion::V1 => false,
            SystemVersion::V2 | SystemVersion::V3 => true,
        }
    }

    pub fn use_root_for_verify_parent_instruction(&self) -> bool {
        match self {
            SystemVersion::V1 | SystemVersion::V2 => true,
            SystemVersion::V3 => false,
        }
    }
}

#[derive(Clone)]
pub enum SystemLockData {
    KeyValueEntry(KeyValueEntryLockData),
    Field(FieldLockData),
    Default,
}

impl Default for SystemLockData {
    fn default() -> Self {
        SystemLockData::Default
    }
}

#[derive(Clone)]
pub enum KeyValueEntryLockData {
    Read,
    KVStoreWrite {
        kv_store_validation_target: KVStoreTypeTarget,
    },
    KVCollectionWrite {
        target: BlueprintTypeTarget,
        collection_index: CollectionIndex,
    },
}

#[derive(Clone)]
pub enum FieldLockData {
    Read,
    Write {
        target: BlueprintTypeTarget,
        field_index: u8,
    },
}

impl SystemLockData {
    pub fn is_kv_entry(&self) -> bool {
        matches!(self, SystemLockData::KeyValueEntry(..))
    }

    pub fn is_kv_entry_with_write(&self) -> bool {
        match self {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::KVCollectionWrite { .. })
            | SystemLockData::KeyValueEntry(KeyValueEntryLockData::KVStoreWrite { .. }) => true,
            _ => false,
        }
    }
}

/// Effectively a trait alias for `KernelApi<CallbackObject = System<Self::SystemCallback, Self::Executable>>`
pub trait SystemBasedKernelApi: KernelApi<CallbackObject = System<Self::SystemCallback>> {
    type SystemCallback: SystemCallbackObject;

    fn system_service(&mut self) -> SystemService<'_, Self> {
        SystemService::new(self)
    }
}

impl<V: SystemCallbackObject, K: KernelApi<CallbackObject = System<V>>> SystemBasedKernelApi for K {
    type SystemCallback = V;
}

/// Effectively a trait alias for `KernelInternalApi<CallbackObject = System<Self::SystemCallback, Self::Executable>>`
pub trait SystemBasedKernelInternalApi:
    KernelInternalApi<System = System<Self::SystemCallback>>
{
    type SystemCallback: SystemCallbackObject;

    fn system_module_api(&mut self) -> SystemModuleApiImpl<Self> {
        SystemModuleApiImpl::new(self)
    }
}

impl<V: SystemCallbackObject, K: KernelInternalApi<System = System<V>>> SystemBasedKernelInternalApi
    for K
{
    type SystemCallback = V;
}

pub struct SystemInit<I: InitializationParameters<For: SystemCallbackObject<Init = I>>> {
    pub self_init: SystemSelfInit,
    pub callback_init: I,
}

impl<I: InitializationParameters<For: SystemCallbackObject<Init = I>>> SystemInit<I> {
    /// This is expected to follow up with a match statement and a call to `v1` / `v2`
    /// etc, to select the correct concrete generic.
    pub fn load(
        substate_db: &impl SubstateDatabase,
        execution_config: ExecutionConfig,
        callback_init: I,
    ) -> Self {
        let system_boot = SystemBoot::load(substate_db, &execution_config);
        let self_init = SystemSelfInit::new(
            execution_config,
            system_boot.system_version(),
            system_boot.into_parameters(),
        );
        Self {
            self_init,
            callback_init,
        }
    }
}

impl<I: InitializationParameters<For: SystemCallbackObject<Init = I>>> InitializationParameters
    for SystemInit<I>
{
    type For = System<I::For>;
}

pub struct SystemSelfInit {
    // These fields only affect side effects and do not affect ledger state execution
    pub enable_kernel_trace: bool,
    pub enable_cost_breakdown: bool,
    pub execution_trace: Option<usize>,
    pub enable_debug_information: bool,

    // Configuration
    pub system_parameters: SystemParameters,
    pub system_logic_version: SystemVersion,
    pub system_overrides: Option<SystemOverrides>,
}

impl SystemSelfInit {
    pub fn new(
        execution_config: ExecutionConfig,
        system_logic_version: SystemVersion,
        system_parameters: SystemParameters,
    ) -> Self {
        Self {
            enable_kernel_trace: execution_config.enable_kernel_trace,
            enable_cost_breakdown: execution_config.enable_cost_breakdown,
            enable_debug_information: execution_config.enable_debug_information,
            execution_trace: execution_config.execution_trace,
            system_overrides: execution_config.system_overrides,
            system_logic_version,
            system_parameters,
        }
    }
}

pub struct System<V: SystemCallbackObject> {
    pub versioned_system_logic: SystemVersion,
    pub callback: V,
    pub blueprint_cache: NonIterMap<CanonicalBlueprintId, Rc<BlueprintDefinition>>,
    pub schema_cache: NonIterMap<SchemaHash, Rc<VersionedScryptoSchema>>,
    pub auth_cache: NonIterMap<CanonicalBlueprintId, AuthConfig>,
    pub modules: SystemModuleMixer,
    pub finalization: SystemFinalization,
}

pub trait HasModules {
    fn modules_mut(&mut self) -> &mut SystemModuleMixer;
}

impl<V: SystemCallbackObject> HasModules for System<V> {
    #[inline]
    fn modules_mut(&mut self) -> &mut SystemModuleMixer {
        &mut self.modules
    }
}

pub struct SystemFinalization {
    pub intent_nullifications: Vec<IntentHashNullification>,
}

impl SystemFinalization {
    pub fn no_nullifications() -> Self {
        Self {
            intent_nullifications: vec![],
        }
    }
}

impl<V: SystemCallbackObject> System<V> {
    pub fn new(
        versioned_system_logic: SystemVersion,
        callback: V,
        modules: SystemModuleMixer,
        finalization: SystemFinalization,
    ) -> Self {
        Self {
            callback,
            blueprint_cache: NonIterMap::new(),
            auth_cache: NonIterMap::new(),
            schema_cache: NonIterMap::new(),
            modules,
            finalization,
            versioned_system_logic,
        }
    }

    fn on_move_node<Y: SystemBasedKernelApi>(
        node_id: &NodeId,
        is_moving_down: bool,
        is_to_barrier: bool,
        destination_blueprint_id: Option<BlueprintId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;

        match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let mut service = SystemService::new(api);
                let definition = service.load_blueprint_definition(
                    object_info.blueprint_info.blueprint_id.package_address,
                    &BlueprintVersionKey {
                        blueprint: object_info
                            .blueprint_info
                            .blueprint_id
                            .blueprint_name
                            .clone(),
                        version: BlueprintVersion::default(),
                    },
                )?;
                if definition.hook_exports.contains_key(&BlueprintHook::OnMove) {
                    api.kernel_invoke(Box::new(KernelInvocation {
                        call_frame_data: Actor::BlueprintHook(BlueprintHookActor {
                            receiver: Some(node_id.clone()),
                            blueprint_id: object_info.blueprint_info.blueprint_id.clone(),
                            hook: BlueprintHook::OnMove,
                        }),
                        args: IndexedScryptoValue::from_typed(&OnMoveInput {
                            is_moving_down,
                            is_to_barrier,
                            destination_blueprint_id,
                        }),
                    }))
                    .map(|_| ())
                } else {
                    Ok(())
                }
            }
            TypeInfoSubstate::KeyValueStore(_)
            | TypeInfoSubstate::GlobalAddressReservation(_)
            | TypeInfoSubstate::GlobalAddressPhantom(_) => Ok(()),
        }
    }
}

impl<V: SystemCallbackObject> System<V> {
    #[cfg(not(feature = "alloc"))]
    fn print_executable(executable: &ExecutableTransaction) {
        println!("{:-^120}", "Executable");
        println!("Intent hash: {}", executable.unique_hash());
        println!("Payload size: {}", executable.payload_size());
        println!(
            "Transaction costing parameters: {:?}",
            executable.costing_parameters()
        );
        println!(
            "Pre-allocated addresses: {:?}",
            executable.pre_allocated_addresses()
        );
        println!("Blobs: {:?}", executable.all_blob_hashes());
        println!("References: {:?}", executable.all_references());
    }

    fn read_epoch_uncosted<S: CommitableSubstateStore>(store: &mut S) -> Option<Epoch> {
        // TODO - Instead of doing a check of the exact epoch, we could do a check in range [X, Y]
        //        Which could allow for better caching of transaction validity over epoch boundaries
        match store.read_substate(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::State.into(),
        ) {
            Some(x) => {
                let substate: FieldSubstate<ConsensusManagerStateFieldPayload> =
                    x.as_typed().unwrap();
                Some(substate.into_payload().into_unique_version().epoch)
            }
            None => None,
        }
    }

    fn validate_epoch_range(
        current_epoch: Epoch,
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
    ) -> Result<(), RejectionReason> {
        if current_epoch < start_epoch_inclusive {
            return Err(RejectionReason::TransactionEpochNotYetValid {
                valid_from: start_epoch_inclusive,
                current_epoch,
            });
        }
        if current_epoch >= end_epoch_exclusive {
            return Err(RejectionReason::TransactionEpochNoLongerValid {
                valid_until: end_epoch_exclusive.previous().unwrap_or(Epoch::zero()),
                current_epoch,
            });
        }

        Ok(())
    }

    fn validate_intent_hash_uncosted<S: CommitableSubstateStore>(
        store: &mut S,
        intent_hash: IntentHash,
        expiry_epoch: Epoch,
    ) -> Result<(), RejectionReason> {
        let substate: FieldSubstate<TransactionTrackerSubstate> = store
            .read_substate(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
            )
            .unwrap()
            .as_typed()
            .unwrap();

        let partition_number = substate
            .into_payload()
            .v1()
            .partition_for_expiry_epoch(expiry_epoch)
            .expect("Transaction tracker should cover all valid epoch ranges");

        let substate = store.read_substate(
            TRANSACTION_TRACKER.as_node_id(),
            PartitionNumber(partition_number),
            &SubstateKey::Map(scrypto_encode(intent_hash.as_hash()).unwrap()),
        );

        match substate {
            Some(value) => {
                let substate: KeyValueEntrySubstate<TransactionStatus> = value.as_typed().unwrap();
                match substate.into_value() {
                    Some(status) => match status.into_v1() {
                        TransactionStatusV1::CommittedSuccess
                        | TransactionStatusV1::CommittedFailure => {
                            return Err(RejectionReason::IntentHashPreviouslyCommitted(
                                intent_hash,
                            ));
                        }
                        TransactionStatusV1::Cancelled => {
                            return Err(RejectionReason::IntentHashPreviouslyCancelled(
                                intent_hash,
                            ));
                        }
                    },
                    None => {}
                }
            }
            None => {}
        }

        Ok(())
    }

    fn determine_result_type(
        interpretation_result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
        fee_reserve: &mut SystemLoanFeeReserve,
    ) -> TransactionResultType {
        // A `SuccessButFeeLoanNotRepaid` error is issued if a transaction finishes before
        // the SYSTEM_LOAN_AMOUNT is reached (which trigger a repay event) and even though
        // enough fee has been locked.
        //
        // Do another `repay` try during finalization to remedy it.
        let final_repay_result = fee_reserve.repay_all();

        match interpretation_result {
            Ok(output) => match final_repay_result {
                Ok(_) => TransactionResultType::Commit(Ok(output)), // success and system loan repaid fully
                Err(e) => {
                    if let Some(abort_reason) = e.abortion() {
                        TransactionResultType::Abort(abort_reason.clone())
                    } else {
                        TransactionResultType::Reject(RejectionReason::SuccessButFeeLoanNotRepaid)
                    }
                }
            },
            Err(e) => match e {
                TransactionExecutionError::BootloadingError(e) => {
                    TransactionResultType::Reject(RejectionReason::BootloadingError(e))
                }
                TransactionExecutionError::RuntimeError(e) => {
                    if let Some(abort_reason) = e.abortion() {
                        TransactionResultType::Abort(abort_reason.clone())
                    } else {
                        if fee_reserve.fully_repaid() {
                            TransactionResultType::Commit(Err(e))
                        } else {
                            TransactionResultType::Reject(
                                RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(e),
                            )
                        }
                    }
                }
            },
        }
    }

    fn finalize_fees_for_commit<S: SubstateDatabase>(
        track: &mut Track<S>,
        fee_reserve: SystemLoanFeeReserve,
        is_success: bool,
    ) -> (
        FeeReserveFinalizationSummary,
        IndexMap<NodeId, Decimal>,
        Vec<(EventTypeIdentifier, Vec<u8>)>,
        CostingParameters,
        TransactionCostingParameters,
    ) {
        let mut events = Vec::<(EventTypeIdentifier, Vec<u8>)>::new();

        // Distribute royalty
        for (recipient, amount) in fee_reserve.royalty_cost_breakdown().clone() {
            let node_id = recipient.vault_id();
            let substate_key = FungibleVaultField::Balance.into();
            let mut vault_balance = track
                .read_substate(&node_id, MAIN_BASE_PARTITION, &substate_key)
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_unique_version();
            vault_balance.put(LiquidFungibleResource::new(amount));
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    node_id,
                    MAIN_BASE_PARTITION,
                    substate_key,
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();
            events.push((
                EventTypeIdentifier(
                    Emitter::Method(node_id, ModuleId::Main),
                    DepositEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&DepositEvent { amount }).unwrap(),
            ));
        }

        // Take fee payments
        let (fee_reserve_finalization, costing_parameters, transaction_costing_parameters) =
            fee_reserve.finalize();
        let mut fee_payments: IndexMap<NodeId, Decimal> = index_map_new();
        let mut required = fee_reserve_finalization.total_cost();
        let mut collected_fees = LiquidFungibleResource::new(Decimal::ZERO);
        for (vault_id, mut locked, contingent) in
            fee_reserve_finalization.locked_fees.iter().cloned().rev()
        {
            let amount = if contingent {
                if is_success {
                    Decimal::min(locked.amount(), required)
                } else {
                    Decimal::zero()
                }
            } else {
                Decimal::min(locked.amount(), required)
            };

            // NOTE: Decimal arithmetic operation safe unwrap.
            // No chance to overflow considering current costing parameters

            // Take fees
            collected_fees.put(locked.take_by_amount(amount).unwrap());
            required = required.checked_sub(amount).unwrap();

            // Refund overpayment
            let mut vault_balance = track
                .read_substate(
                    &vault_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::Balance.into(),
                )
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_unique_version();
            vault_balance.put(locked);
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    vault_id,
                    MAIN_BASE_PARTITION,
                    FungibleVaultField::Balance.into(),
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            // Record final payments
            let entry = fee_payments.entry(vault_id).or_default();
            *entry = entry.checked_add(amount).unwrap();

            events.push((
                EventTypeIdentifier(
                    Emitter::Method(vault_id, ModuleId::Main),
                    PayFeeEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&PayFeeEvent { amount }).unwrap(),
            ));
        }
        // Free credit is locked first and thus used last
        let free_credit = transaction_costing_parameters.free_credit_in_xrd;
        if free_credit.is_positive() {
            let amount = Decimal::min(free_credit, required);
            collected_fees.put(LiquidFungibleResource::new(amount));
            required = required.checked_sub(amount).unwrap();
        }

        let to_proposer = fee_reserve_finalization.to_proposer_amount();
        let to_validator_set = fee_reserve_finalization.to_validator_set_amount();
        let to_burn = fee_reserve_finalization.to_burn_amount();

        // Sanity checks
        assert!(
            fee_reserve_finalization.total_bad_debt_in_xrd == Decimal::ZERO,
            "Bad debt is non-zero: {}",
            fee_reserve_finalization.total_bad_debt_in_xrd
        );
        assert!(
            required == Decimal::ZERO,
            "Locked fee does not cover transaction cost: {} required",
            required
        );
        let remaining_collected_fees = collected_fees.amount().checked_sub(fee_reserve_finalization.total_royalty_cost_in_xrd /* royalty already distributed */).unwrap();
        let to_distribute = to_proposer
            .checked_add(to_validator_set)
            .unwrap()
            .checked_add(to_burn)
            .unwrap();
        assert!(
            remaining_collected_fees  == to_distribute,
            "Remaining collected fee isn't equal to amount to distribute (proposer/validator set/burn): {} != {}",
            remaining_collected_fees,
            to_distribute,
        );

        if !to_proposer.is_zero() || !to_validator_set.is_zero() {
            // Fetch current leader
            // TODO: maybe we should move current leader into validator rewards?
            let substate: FieldSubstate<ConsensusManagerStateFieldPayload> = track
                .read_substate(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::State.into(),
                )
                .unwrap()
                .as_typed()
                .unwrap();
            let current_leader = substate.into_payload().into_unique_version().current_leader;

            // Update validator rewards
            let substate: FieldSubstate<ConsensusManagerValidatorRewardsFieldPayload> = track
                .read_substate(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::ValidatorRewards.into(),
                )
                .unwrap()
                .as_typed()
                .unwrap();

            let mut rewards = substate.into_payload().into_unique_version();

            if let Some(current_leader) = current_leader {
                let entry = rewards.proposer_rewards.entry(current_leader).or_default();
                *entry = entry.checked_add(to_proposer).unwrap()
            } else {
                // If there is no current leader, the rewards go to the pool
            };
            let vault_node_id = rewards.rewards_vault.0 .0;

            track
                .set_substate(
                    CONSENSUS_MANAGER.into_node_id(),
                    MAIN_BASE_PARTITION,
                    ConsensusManagerField::ValidatorRewards.into(),
                    IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(
                        ConsensusManagerValidatorRewardsFieldPayload::from_content_source(rewards),
                    )),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            // Put validator rewards into the vault
            let total_amount = to_proposer.checked_add(to_validator_set).unwrap();
            let mut vault_balance = track
                .read_substate(
                    &vault_node_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::Balance.into(),
                )
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_unique_version();
            vault_balance.put(collected_fees.take_by_amount(total_amount).unwrap());
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    vault_node_id,
                    MAIN_BASE_PARTITION,
                    FungibleVaultField::Balance.into(),
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            events.push((
                EventTypeIdentifier(
                    Emitter::Method(vault_node_id, ModuleId::Main),
                    DepositEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&DepositEvent {
                    amount: total_amount,
                })
                .unwrap(),
            ));
        }

        if to_burn.is_positive() {
            events.push((
                EventTypeIdentifier(
                    Emitter::Method(XRD.into_node_id(), ModuleId::Main),
                    "BurnFungibleResourceEvent".to_string(),
                ),
                scrypto_encode(&BurnFungibleResourceEvent { amount: to_burn }).unwrap(),
            ));
        }

        (
            fee_reserve_finalization,
            fee_payments,
            events,
            costing_parameters,
            transaction_costing_parameters,
        )
    }

    fn update_transaction_tracker<S: SubstateDatabase>(
        track: &mut Track<S>,
        next_epoch: Epoch,
        intent_hash_nullifications: Vec<IntentHashNullification>,
        is_success: bool,
    ) -> Vec<Nullification> {
        // NOTE: In the case of system transactions, we could skip most of this...
        //       except for backwards compatibility, we can't!

        // Read the intent hash store
        let mut transaction_tracker = track
            .read_substate(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
            )
            .unwrap()
            .as_typed::<FieldSubstate<TransactionTrackerSubstate>>()
            .unwrap()
            .into_payload()
            .into_v1();

        let mut performed_nullifications = vec![];

        for intent_hash_nullification in intent_hash_nullifications {
            let Some(nullification) = Nullification::of_intent(
                intent_hash_nullification,
                Epoch::of(transaction_tracker.start_epoch),
                is_success,
            ) else {
                continue;
            };
            let (expiry_epoch, hash) = nullification.transaction_tracker_keys();
            performed_nullifications.push(nullification);

            let partition_number = transaction_tracker.partition_for_expiry_epoch(expiry_epoch)
                .expect("Validation of the max expiry epoch window combined with the current epoch check on launch should ensure that the expiry epoch is in range for the transaction tracker");

            // Update the status of the intent hash
            track
                .set_substate(
                    TRANSACTION_TRACKER.into_node_id(),
                    PartitionNumber(partition_number),
                    SubstateKey::Map(scrypto_encode(&hash).unwrap()),
                    IndexedScryptoValue::from_typed(&KeyValueEntrySubstate::V1(
                        KeyValueEntrySubstateV1 {
                            value: Some(if is_success {
                                TransactionStatus::V1(TransactionStatusV1::CommittedSuccess)
                            } else {
                                TransactionStatus::V1(TransactionStatusV1::CommittedFailure)
                            }),
                            // TODO: maybe make it immutable, but how does this affect partition deletion?
                            lock_status: LockStatus::Unlocked,
                        },
                    )),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();
        }

        // Check if all intent hashes in the first epoch have expired, based on the `next_epoch`.
        //
        // In this particular implementation, because the transaction tracker coverage is greater than
        // the max epoch range in transaction header, we must check epoch range first to
        // ensure we don't store intent hash too far into the future.
        //
        // Also, we need to make sure epoch doesn't jump by a large distance.
        if next_epoch.number()
            >= transaction_tracker.start_epoch + transaction_tracker.epochs_per_partition
        {
            let discarded_partition = transaction_tracker.advance();
            track.delete_partition(
                TRANSACTION_TRACKER.as_node_id(),
                PartitionNumber(discarded_partition),
            );
        }
        track
            .set_substate(
                TRANSACTION_TRACKER.into_node_id(),
                MAIN_BASE_PARTITION,
                TransactionTrackerField::TransactionTracker.into(),
                IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(
                    TransactionTrackerSubstate::V1(transaction_tracker),
                )),
                &mut |_| -> Result<(), ()> { Ok(()) },
            )
            .unwrap();

        performed_nullifications
    }

    #[cfg(not(feature = "alloc"))]
    fn print_execution_summary(receipt: &TransactionReceipt) {
        // NB - we use "to_string" to ensure they align correctly

        if let Some(fee_details) = &receipt.fee_details {
            println!("{:-^120}", "Execution Cost Breakdown");
            for (k, v) in &fee_details.execution_cost_breakdown {
                println!("{:<75}: {:>25}", k, v.to_string());
            }

            println!("{:-^120}", "Finalization Cost Breakdown");
            for (k, v) in &fee_details.finalization_cost_breakdown {
                println!("{:<75}: {:>25}", k, v.to_string());
            }
        }

        println!("{:-^120}", "Fee Summary");
        println!(
            "{:<40}: {:>25}",
            "Execution Cost Units Consumed",
            receipt
                .fee_summary
                .total_execution_cost_units_consumed
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Finalization Cost Units Consumed",
            receipt
                .fee_summary
                .total_finalization_cost_units_consumed
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Execution Cost in XRD",
            receipt.fee_summary.total_execution_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Finalization Cost in XRD",
            receipt
                .fee_summary
                .total_finalization_cost_in_xrd
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Tipping Cost in XRD",
            receipt.fee_summary.total_tipping_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Storage Cost in XRD",
            receipt.fee_summary.total_storage_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Royalty Costs in XRD",
            receipt.fee_summary.total_royalty_cost_in_xrd.to_string()
        );

        match &receipt.result {
            TransactionResult::Commit(commit) => {
                println!("{:-^120}", "Application Logs");
                for (level, message) in &commit.application_logs {
                    println!("[{}] {}", level, message);
                }

                println!("{:-^120}", "Outcome");
                println!(
                    "{}",
                    match &commit.outcome {
                        TransactionOutcome::Success(_) => "Success".to_string(),
                        TransactionOutcome::Failure(error) => format!("Failure: {:?}", error),
                    }
                );
            }
            TransactionResult::Reject(e) => {
                println!("{:-^120}", "Transaction Rejected");
                println!("{:?}", e.reason);
            }
            TransactionResult::Abort(e) => {
                println!("{:-^120}", "Transaction Aborted");
                println!("{:?}", e);
            }
        }
        println!("{:-^120}", "Finish");
    }

    fn reference_check(
        references: &IndexSet<Reference>,
        modules: &mut SystemModuleMixer,
        store: &mut impl CommitableSubstateStore,
        always_visible_global_nodes: &IndexSet<NodeId>,
    ) -> Result<(IndexSet<GlobalAddress>, IndexSet<InternalAddress>), BootloadingError> {
        let mut global_addresses = indexset!();
        let mut direct_accesses = indexset!();

        // Check references
        for reference in references.iter() {
            let node_id = &reference.0;

            if always_visible_global_nodes.contains(node_id) {
                // Allow always visible node and do not add reference
                continue;
            }

            if node_id.is_global_preallocated() {
                // Allow global virtual and add reference
                global_addresses.insert(GlobalAddress::new_or_panic(node_id.clone().into()));
                continue;
            }

            let ref_value = store
                .read_substate(
                    node_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .ok_or_else(|| BootloadingError::ReferencedNodeDoesNotExist((*node_id).into()))?;

            match Self::verify_boot_ref_value(modules, node_id, ref_value)? {
                StableReferenceType::Global => {
                    global_addresses.insert(GlobalAddress::new_or_panic(node_id.clone().into()));
                }
                StableReferenceType::DirectAccess => {
                    direct_accesses.insert(InternalAddress::new_or_panic(node_id.clone().into()));
                }
            }
        }

        Ok((global_addresses, direct_accesses))
    }

    /// Checks that references exist in the store
    fn build_call_frame_inits_with_reference_check<'a>(
        intents: impl Iterator<Item = &'a ExecutableIntent>,
        modules: &mut SystemModuleMixer,
        store: &mut impl CommitableSubstateStore,
        always_visible_global_nodes: &'static IndexSet<NodeId>,
    ) -> Result<Vec<CallFrameInit<Actor>>, BootloadingError> {
        let mut init_call_frames = vec![];
        for (index, intent) in intents.enumerate() {
            let (global_addresses, direct_accesses) = Self::reference_check(
                &intent.references,
                modules,
                store,
                always_visible_global_nodes,
            )?;

            init_call_frames.push(CallFrameInit {
                data: Actor::Root,
                global_addresses,
                direct_accesses,
                always_visible_global_nodes,
                stack_id: index,
            });
        }

        Ok(init_call_frames)
    }

    fn verify_boot_ref_value(
        modules: &mut SystemModuleMixer,
        node_id: &NodeId,
        ref_value: &IndexedScryptoValue,
    ) -> Result<StableReferenceType, BootloadingError> {
        if let Some(costing) = modules.costing_mut() {
            let io_access = IOAccess::ReadFromDb(
                CanonicalSubstateKey {
                    node_id: *node_id,
                    partition_number: TYPE_INFO_FIELD_PARTITION,
                    substate_key: SubstateKey::Field(TypeInfoField::TypeInfo.field_index()),
                },
                ref_value.len(),
            );
            let event = CheckReferenceEvent::IOAccess(&io_access);

            costing
                .apply_deferred_execution_cost(ExecutionCostingEntry::CheckReference {
                    event: &event,
                })
                .map_err(|e| BootloadingError::FailedToApplyDeferredCosts(e))?;
        }

        let type_substate: TypeInfoSubstate = ref_value.as_typed().unwrap();
        return match &type_substate {
            TypeInfoSubstate::Object(
                info @ ObjectInfo {
                    blueprint_info: BlueprintInfo { blueprint_id, .. },
                    ..
                },
            ) => {
                if info.is_global() {
                    Ok(StableReferenceType::Global)
                } else if blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                    && (blueprint_id.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                        || blueprint_id.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                {
                    Ok(StableReferenceType::DirectAccess)
                } else {
                    Err(BootloadingError::ReferencedNodeDoesNotAllowDirectAccess(
                        (*node_id).into(),
                    ))
                }
            }
            _ => Err(BootloadingError::ReferencedNodeIsNotAnObject(
                (*node_id).into(),
            )),
        };
    }

    fn create_non_commit_receipt(
        result: TransactionResult,
        print_execution_summary: bool,
        costing_module: CostingModule,
    ) -> TransactionReceipt {
        let (fee_reserve, cost_breakdown, detailed_cost_breakdown) =
            costing_module.unpack_for_receipt();
        let (finalization_summary, costing_parameters, transaction_costing_parameters) =
            fee_reserve.finalize();
        let fee_summary = finalization_summary.into();

        Self::create_receipt_internal(
            print_execution_summary,
            costing_parameters,
            cost_breakdown,
            detailed_cost_breakdown,
            transaction_costing_parameters,
            fee_summary,
            result,
        )
    }

    fn create_rejection_receipt(
        reason: impl Into<RejectionReason>,
        modules: SystemModuleMixer,
    ) -> TransactionReceipt {
        Self::create_non_commit_receipt(
            TransactionResult::Reject(RejectResult {
                reason: reason.into(),
            }),
            modules.is_kernel_trace_enabled(),
            modules.unpack_costing(),
        )
    }

    fn create_abort_receipt(
        reason: impl Into<AbortReason>,
        modules: SystemModuleMixer,
    ) -> TransactionReceipt {
        Self::create_non_commit_receipt(
            TransactionResult::Abort(AbortResult {
                reason: reason.into(),
            }),
            modules.is_kernel_trace_enabled(),
            modules.unpack_costing(),
        )
    }

    fn create_commit_receipt<S: SubstateDatabase>(
        outcome: Result<Vec<InstructionOutput>, RuntimeError>,
        mut track: Track<S>,
        modules: SystemModuleMixer,
        system_finalization: SystemFinalization,
    ) -> TransactionReceipt {
        let print_execution_summary = modules.is_kernel_trace_enabled();
        let execution_trace_enabled = modules.is_execution_trace_enabled();
        let (costing_module, runtime_module, execution_trace_module) = modules.unpack();
        let (mut fee_reserve, cost_breakdown, detailed_cost_breakdown) =
            costing_module.unpack_for_receipt();
        let is_success = outcome.is_ok();

        // Commit/revert
        if !is_success {
            fee_reserve.revert_royalty();
            track.revert_non_force_write_changes();
        }

        // Distribute fees
        let (
            fee_reserve_finalization,
            paying_vaults,
            finalization_events,
            costing_parameters,
            transaction_costing_parameters,
        ) = Self::finalize_fees_for_commit(&mut track, fee_reserve, is_success);

        let fee_destination = FeeDestination {
            to_proposer: fee_reserve_finalization.to_proposer_amount(),
            to_validator_set: fee_reserve_finalization.to_validator_set_amount(),
            to_burn: fee_reserve_finalization.to_burn_amount(),
            to_royalty_recipients: fee_reserve_finalization.royalty_cost_breakdown.clone(),
        };

        // Update intent hash status
        let performed_nullifications =
            if let Some(next_epoch) = Self::read_epoch_uncosted(&mut track) {
                Self::update_transaction_tracker(
                    &mut track,
                    next_epoch,
                    system_finalization.intent_nullifications,
                    is_success,
                )
            } else {
                vec![]
            };

        // Finalize events and logs
        let (mut application_events, application_logs) = runtime_module.finalize(is_success);
        application_events.extend(finalization_events);

        // Finalize track
        let (tracked_substates, substate_db) = {
            match track.finalize() {
                Ok(result) => result,
                Err(TrackFinalizeError::TransientSubstateOwnsNode) => {
                    panic!("System invariants should prevent transient substate from owning nodes");
                }
            }
        };

        // Generate state updates from tracked substates
        // Note that this process will prune invalid reads
        let (new_node_ids, state_updates) = tracked_substates.to_state_updates();

        // Summarizes state updates
        let system_structure =
            SystemStructure::resolve(substate_db, &state_updates, &application_events);
        let state_update_summary =
            StateUpdateSummary::new(substate_db, new_node_ids, &state_updates);

        // Resource reconciliation does not currently work in preview mode
        if transaction_costing_parameters.free_credit_in_xrd.is_zero() {
            reconcile_resource_state_and_events(
                &state_update_summary,
                &application_events,
                SystemDatabaseReader::new_with_overlay(substate_db, &state_updates),
            );
        }

        let execution_trace = if execution_trace_enabled {
            Some(execution_trace_module.finalize(&paying_vaults, is_success))
        } else {
            None
        };

        let fee_summary = fee_reserve_finalization.into();
        let result = TransactionResult::Commit(CommitResult {
            state_updates,
            state_update_summary,
            fee_source: FeeSource { paying_vaults },
            fee_destination,
            outcome: match outcome {
                Ok(o) => TransactionOutcome::Success(o),
                Err(e) => TransactionOutcome::Failure(e),
            },
            application_events,
            application_logs,
            system_structure,
            execution_trace,
            performed_nullifications,
        });

        Self::create_receipt_internal(
            print_execution_summary,
            costing_parameters,
            cost_breakdown,
            detailed_cost_breakdown,
            transaction_costing_parameters,
            fee_summary,
            result,
        )
    }

    fn create_receipt_internal(
        print_execution_summary: bool,
        costing_parameters: CostingParameters,
        cost_breakdown: Option<CostBreakdown>,
        detailed_cost_breakdown: Option<DetailedCostBreakdown>,
        transaction_costing_parameters: TransactionCostingParameters,
        fee_summary: TransactionFeeSummary,
        result: TransactionResult,
    ) -> TransactionReceipt {
        let transaction_costing_parameters = TransactionCostingParametersReceiptV2 {
            tip_proportion: transaction_costing_parameters.tip.proportion(),
            free_credit_in_xrd: transaction_costing_parameters.free_credit_in_xrd,
        };

        let fee_details = cost_breakdown.map(|b| TransactionFeeDetails {
            execution_cost_breakdown: b.execution_cost_breakdown.into_iter().collect(),
            finalization_cost_breakdown: b.finalization_cost_breakdown.into_iter().collect(),
        });

        let debug_information = detailed_cost_breakdown.map(|b| TransactionDebugInformation {
            detailed_execution_cost_breakdown: b.detailed_execution_cost_breakdown,
        });

        let receipt = TransactionReceipt {
            costing_parameters,
            transaction_costing_parameters,
            fee_summary,
            fee_details,
            result,
            resources_usage: None,
            debug_information,
        };

        // Dump summary
        #[cfg(not(feature = "alloc"))]
        if print_execution_summary {
            Self::print_execution_summary(&receipt);
        }

        receipt
    }

    fn resolve_modules(
        executable: &ExecutableTransaction,
        init_input: SystemSelfInit,
    ) -> Result<SystemModuleMixer, TransactionReceiptV1> {
        let mut system_parameters = init_input.system_parameters;
        let system_logic_version = init_input.system_logic_version;

        let mut enabled_modules = {
            let mut enabled_modules = EnabledModules::AUTH | EnabledModules::TRANSACTION_RUNTIME;
            if !executable.disable_limits_and_costing_modules() {
                enabled_modules |= EnabledModules::LIMITS;
                enabled_modules |= EnabledModules::COSTING;
            };

            if init_input.enable_kernel_trace {
                enabled_modules |= EnabledModules::KERNEL_TRACE;
            }
            if init_input.execution_trace.is_some() {
                enabled_modules |= EnabledModules::EXECUTION_TRACE;
            }

            enabled_modules
        };

        let mut abort_when_loan_repaid = false;

        // Override system configuration
        if let Some(system_overrides) = &init_input.system_overrides {
            if let Some(costing_override) = &system_overrides.costing_parameters {
                system_parameters.costing_parameters = costing_override.clone();
            }

            if let Some(limits_override) = &system_overrides.limit_parameters {
                system_parameters.limit_parameters = limits_override.clone();
            }

            if let Some(network_definition) = &system_overrides.network_definition {
                system_parameters.network_definition = network_definition.clone();
            }

            if system_overrides.disable_auth {
                enabled_modules.remove(EnabledModules::AUTH);
            }

            if system_overrides.disable_costing {
                enabled_modules.remove(EnabledModules::COSTING);
            }

            if system_overrides.disable_limits {
                enabled_modules.remove(EnabledModules::LIMITS);
            }

            if system_overrides.abort_when_loan_repaid {
                abort_when_loan_repaid = true;
            }
        }

        let costing_module = CostingModule {
            current_depth: 0,
            fee_reserve: SystemLoanFeeReserve::new(
                system_parameters.costing_parameters,
                executable.costing_parameters().clone(),
                abort_when_loan_repaid,
            ),
            fee_table: FeeTable::new(system_logic_version),
            tx_payload_len: executable.payload_size(),
            tx_num_of_signature_validations: executable.num_of_signature_validations(),
            config: system_parameters.costing_module_config,
            cost_breakdown: if init_input.enable_cost_breakdown {
                Some(Default::default())
            } else {
                None
            },
            detailed_cost_breakdown: if init_input.enable_debug_information {
                Some(Default::default())
            } else {
                None
            },
            on_apply_cost: Default::default(),
        };

        let auth_module = system_logic_version
            .create_auth_module(&executable)
            .map_err(|reason| {
                let print_execution_summary =
                    enabled_modules.contains(EnabledModules::KERNEL_TRACE);
                Self::create_non_commit_receipt(
                    TransactionResult::Reject(RejectResult { reason }),
                    print_execution_summary,
                    costing_module.clone(),
                )
            })?;

        let module_mixer = SystemModuleMixer::new(
            enabled_modules,
            KernelTraceModule,
            TransactionRuntimeModule::new(
                system_parameters.network_definition,
                *executable.unique_hash(),
            ),
            auth_module,
            LimitsModule::from_params(system_parameters.limit_parameters),
            costing_module,
            ExecutionTraceModule::new(init_input.execution_trace.unwrap_or(0)),
        );

        Ok(module_mixer)
    }
}

impl<V: SystemCallbackObject> KernelTransactionExecutor for System<V> {
    type Init = SystemInit<V::Init>;
    type Executable = ExecutableTransaction;
    type ExecutionOutput = Vec<InstructionOutput>;
    type Receipt = TransactionReceipt;

    fn init(
        store: &mut impl CommitableSubstateStore,
        executable: &ExecutableTransaction,
        init_input: Self::Init,
        always_visible_global_nodes: &'static IndexSet<NodeId>,
    ) -> Result<(Self, Vec<CallFrameInit<Actor>>), Self::Receipt> {
        // Dump executable
        #[cfg(not(feature = "alloc"))]
        if init_input.self_init.enable_kernel_trace {
            Self::print_executable(executable);
        }

        let logic_version = init_input.self_init.system_logic_version;
        let mut modules = Self::resolve_modules(executable, init_input.self_init)?;

        // NOTE: Have to use match pattern rather than map_err to appease the borrow checker
        let callback = match V::init(init_input.callback_init) {
            Ok(callback) => callback,
            Err(error) => return Err(Self::create_rejection_receipt(error, modules)),
        };

        match modules.init() {
            Ok(()) => {}
            Err(error) => return Err(Self::create_rejection_receipt(error, modules)),
        }

        // Perform runtime validation.
        // NOTE: The epoch doesn't exist yet on the very first transaction, so we skip this
        if let Some(current_epoch) = Self::read_epoch_uncosted(store) {
            // We are assuming that intent hash store is ready when epoch manager is ready.
            if let Some(range) = executable.overall_epoch_range() {
                let epoch_validation_result = Self::validate_epoch_range(
                    current_epoch,
                    range.start_epoch_inclusive,
                    range.end_epoch_exclusive,
                );
                match epoch_validation_result {
                    Ok(()) => {}
                    Err(error) => return Err(Self::create_rejection_receipt(error, modules)),
                }
            }
        }

        for hash_nullification in executable.intent_hash_nullifications() {
            let intent_hash_validation_result = match hash_nullification {
                IntentHashNullification::TransactionIntent {
                    intent_hash,
                    expiry_epoch,
                } => Self::validate_intent_hash_uncosted(
                    store,
                    IntentHash::Transaction(*intent_hash),
                    *expiry_epoch,
                ),
                IntentHashNullification::SimulatedTransactionIntent { .. } => {
                    // No validation is done on a simulated transaction intent used during preview
                    Ok(())
                }
                IntentHashNullification::Subintent {
                    intent_hash,
                    expiry_epoch,
                } => Self::validate_intent_hash_uncosted(
                    store,
                    IntentHash::Subintent(*intent_hash),
                    *expiry_epoch,
                ),
                IntentHashNullification::SimulatedSubintent { .. } => {
                    // No validation is done on a simulated subintent used during preview
                    Ok(())
                }
            }
            .and_then(|_| {
                let charge_for_nullification_check = match hash_nullification {
                    IntentHashNullification::TransactionIntent { .. }
                    | IntentHashNullification::SimulatedTransactionIntent { .. } => {
                        logic_version.should_charge_for_transaction_intent()
                    }
                    IntentHashNullification::Subintent { .. }
                    | IntentHashNullification::SimulatedSubintent { .. } => true,
                };

                if charge_for_nullification_check {
                    if let Some(costing) = modules.costing_mut() {
                        return costing
                            .apply_deferred_execution_cost(
                                ExecutionCostingEntry::CheckIntentValidity,
                            )
                            .map_err(|e| {
                                RejectionReason::BootloadingError(
                                    BootloadingError::FailedToApplyDeferredCosts(e),
                                )
                            });
                    }
                }

                Ok(())
            });

            match intent_hash_validation_result {
                Ok(()) => {}
                Err(error) => return Err(Self::create_rejection_receipt(error, modules)),
            }
        }

        if let Some(range) = executable.overall_proposer_timestamp_range() {
            if range.start_timestamp_inclusive.is_some() || range.end_timestamp_exclusive.is_some()
            {
                let substate: ConsensusManagerProposerMilliTimestampFieldSubstate = store
                    .read_substate(
                        CONSENSUS_MANAGER.as_node_id(),
                        MAIN_BASE_PARTITION,
                        &ConsensusManagerField::ProposerMilliTimestamp.into(),
                    )
                    .unwrap()
                    .as_typed()
                    .unwrap();
                let current_time = Instant::new(
                    substate
                        .into_payload()
                        .fully_update_and_into_latest_version()
                        .epoch_milli
                        / 1000,
                );
                if let Some(start_timestamp_inclusive) = range.start_timestamp_inclusive {
                    if current_time < start_timestamp_inclusive {
                        return Err(Self::create_rejection_receipt(
                            RejectionReason::TransactionProposerTimestampNotYetValid {
                                valid_from_inclusive: start_timestamp_inclusive,
                                current_time,
                            },
                            modules,
                        ));
                    }
                }

                if let Some(end_timestamp_exclusive) = range.end_timestamp_exclusive {
                    if current_time >= end_timestamp_exclusive {
                        return Err(Self::create_rejection_receipt(
                            RejectionReason::TransactionProposerTimestampNoLongerValid {
                                valid_to_exclusive: end_timestamp_exclusive,
                                current_time,
                            },
                            modules,
                        ));
                    }
                }

                if let Some(costing) = modules.costing_mut() {
                    if let Err(error) =
                        costing.apply_deferred_execution_cost(ExecutionCostingEntry::CheckTimestamp)
                    {
                        return Err(Self::create_rejection_receipt(
                            RejectionReason::BootloadingError(
                                BootloadingError::FailedToApplyDeferredCosts(error),
                            ),
                            modules,
                        ));
                    }
                }
            }
        }

        let call_frame_inits = match Self::build_call_frame_inits_with_reference_check(
            executable.all_intents(),
            &mut modules,
            store,
            always_visible_global_nodes,
        ) {
            Ok(call_frame_inits) => call_frame_inits,
            Err(error) => return Err(Self::create_rejection_receipt(error, modules)),
        };

        let system = System::new(
            logic_version,
            callback,
            modules,
            SystemFinalization {
                intent_nullifications: executable
                    .intent_hash_nullifications()
                    .iter()
                    .cloned()
                    .collect(),
            },
        );

        Ok((system, call_frame_inits))
    }

    fn execute<Y: SystemBasedKernelApi>(
        api: &mut Y,
        executable: &ExecutableTransaction,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let mut system_service = SystemService::new(api);

        // Allocate global addresses
        let mut global_address_reservations = Vec::new();
        for PreAllocatedAddress {
            blueprint_id,
            address,
        } in executable.pre_allocated_addresses()
        {
            let global_address_reservation =
                system_service.prepare_global_address(blueprint_id.clone(), address.clone())?;
            global_address_reservations.push(global_address_reservation);
        }

        let system_logic_version = system_service.system().versioned_system_logic;

        let output = system_logic_version.execute_transaction(
            api,
            executable,
            global_address_reservations,
        )?;

        Ok(output)
    }

    fn finalize(
        &mut self,
        executable: &ExecutableTransaction,
        info: StoreCommitInfo,
    ) -> Result<(), RuntimeError> {
        self.modules.on_teardown()?;

        // Note that if a transactions fails during this phase, the costing is
        // done as if it would succeed.
        for store_commit in &info {
            self.modules
                .apply_finalization_cost(FinalizationCostingEntry::CommitStateUpdates {
                    store_commit,
                })
                .map_err(|e| RuntimeError::FinalizationCostingError(e))?;
        }
        self.modules
            .apply_finalization_cost(FinalizationCostingEntry::CommitEvents {
                events: &self.modules.events().clone(),
            })
            .map_err(|e| RuntimeError::FinalizationCostingError(e))?;
        self.modules
            .apply_finalization_cost(FinalizationCostingEntry::CommitLogs {
                logs: &self.modules.logs().clone(),
            })
            .map_err(|e| RuntimeError::FinalizationCostingError(e))?;
        let num_of_intent_statuses = executable
            .intent_hash_nullifications()
            .iter()
            .map(|n| match n {
                IntentHashNullification::TransactionIntent { .. }
                | IntentHashNullification::SimulatedTransactionIntent { .. } => {
                    if self
                        .versioned_system_logic
                        .should_charge_for_transaction_intent()
                    {
                        1
                    } else {
                        0
                    }
                }
                IntentHashNullification::Subintent { .. }
                | IntentHashNullification::SimulatedSubintent { .. } => 1,
            })
            .sum();
        self.modules
            .apply_finalization_cost(FinalizationCostingEntry::CommitIntentStatus {
                num_of_intent_statuses,
            })
            .map_err(|e| RuntimeError::FinalizationCostingError(e))?;

        /* state storage costs */
        for store_commit in &info {
            self.modules
                .apply_storage_cost(StorageType::State, store_commit.len_increase())
                .map_err(|e| RuntimeError::FinalizationCostingError(e))?;
        }

        /* archive storage costs */
        let total_event_size = self.modules.events().iter().map(|x| x.len()).sum();
        self.modules
            .apply_storage_cost(StorageType::Archive, total_event_size)
            .map_err(|e| RuntimeError::FinalizationCostingError(e))?;

        let total_log_size = self.modules.logs().iter().map(|x| x.1.len()).sum();
        self.modules
            .apply_storage_cost(StorageType::Archive, total_log_size)
            .map_err(|e| RuntimeError::FinalizationCostingError(e))?;

        Ok(())
    }

    fn create_receipt<S: SubstateDatabase>(
        mut self,
        track: Track<S>,
        interpretation_result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
    ) -> TransactionReceipt {
        // Panic if an error is encountered in the system layer or below. The following code
        // is only enabled when compiling with the standard library since the panic catching
        // machinery and `SystemPanic` errors are only implemented in `std`.
        #[cfg(feature = "std")]
        if let Err(TransactionExecutionError::RuntimeError(RuntimeError::SystemError(
            SystemError::SystemPanic(..),
        ))) = interpretation_result
        {
            panic!("An error has occurred in the system layer or below and thus the transaction executor has panicked. Error: \"{interpretation_result:?}\"")
        }

        #[cfg(not(feature = "alloc"))]
        if self.modules.is_kernel_trace_enabled() {
            println!("{:-^120}", "Interpretation Results");
            println!("{:?}", interpretation_result);
        }

        let result_type = Self::determine_result_type(
            interpretation_result,
            &mut self.modules.costing_mut_even_if_disabled().fee_reserve,
        );

        match result_type {
            TransactionResultType::Reject(reason) => {
                Self::create_rejection_receipt(reason, self.modules)
            }
            TransactionResultType::Abort(reason) => {
                Self::create_abort_receipt(reason, self.modules)
            }
            TransactionResultType::Commit(outcome) => {
                Self::create_commit_receipt(outcome, track, self.modules, self.finalization)
            }
        }
    }
}

impl<V: SystemCallbackObject> KernelCallbackObject for System<V> {
    type LockData = SystemLockData;
    type CallFrameData = Actor;

    fn on_pin_node<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_pin_node(api, node_id)
    }

    fn on_create_node<Y: KernelInternalApi<System = Self>>(
        event: CreateNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_create_node(api, &event)
    }

    fn on_drop_node<Y: KernelInternalApi<System = Self>>(
        event: DropNodeEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_drop_node(api, &event)
    }

    fn on_move_module<Y: KernelInternalApi<System = Self>>(
        event: MoveModuleEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_move_module(api, &event)
    }

    fn on_open_substate<Y: KernelInternalApi<System = Self>>(
        event: OpenSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_open_substate(api, &event)
    }

    fn on_close_substate<Y: KernelInternalApi<System = Self>>(
        event: CloseSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_close_substate(api, &event)
    }

    fn on_read_substate<Y: KernelInternalApi<System = Self>>(
        event: ReadSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_read_substate(api, &event)
    }

    fn on_write_substate<Y: KernelInternalApi<System = Self>>(
        event: WriteSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_write_substate(api, &event)
    }

    fn on_set_substate<Y: KernelInternalApi<System = Self>>(
        event: SetSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_set_substate(api, &event)
    }

    fn on_remove_substate<Y: KernelInternalApi<System = Self>>(
        event: RemoveSubstateEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_remove_substate(api, &event)
    }

    fn on_scan_keys<Y: KernelInternalApi<System = Self>>(
        event: ScanKeysEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_scan_keys(api, &event)
    }

    fn on_drain_substates<Y: KernelInternalApi<System = Self>>(
        event: DrainSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_drain_substates(api, &event)
    }

    fn on_scan_sorted_substates<Y: KernelInternalApi<System = Self>>(
        event: ScanSortedSubstatesEvent,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_scan_sorted_substates(api, &event)
    }

    fn before_invoke<Y: KernelApi<CallbackObject = Self>>(
        invocation: &KernelInvocation<Actor>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let is_to_barrier = invocation.call_frame_data.is_barrier();
        let destination_blueprint_id = invocation.call_frame_data.blueprint_id();

        for node_id in invocation.args.owned_nodes() {
            Self::on_move_node(
                node_id,
                true,
                is_to_barrier,
                destination_blueprint_id.clone(),
                api,
            )?;
        }

        SystemModuleMixer::before_invoke(api, invocation)
    }

    fn on_execution_start<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_execution_start(api)
    }

    fn invoke_upstream<Y: KernelApi<CallbackObject = Self>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let mut system = SystemService::new(api);
        let actor = system.current_actor();
        let node_id = actor.node_id();
        let is_direct_access = actor.is_direct_access();

        // Make dependent resources/components visible
        if let Some(blueprint_id) = actor.blueprint_id() {
            let key = BlueprintVersionKey {
                blueprint: blueprint_id.blueprint_name.clone(),
                version: BlueprintVersion::default(),
            };

            let handle = system.kernel_open_substate_with_default(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&key).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry = KeyValueEntrySubstate::<()>::default();
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;
            system.kernel_read_substate(handle)?;
            system.kernel_close_substate(handle)?;
        }

        match &actor {
            Actor::Root => panic!("Root is invoked"),
            actor @ Actor::Method(MethodActor { ident, .. })
            | actor @ Actor::Function(FunctionActor { ident, .. }) => {
                let blueprint_id = actor.blueprint_id().unwrap();

                //  Validate input
                let definition = system.load_blueprint_definition(
                    blueprint_id.package_address,
                    &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                )?;

                let target = system.get_actor_type_target()?;

                // Validate input
                system.validate_blueprint_payload(
                    &target,
                    BlueprintPayloadIdentifier::Function(ident.clone(), InputOrOutput::Input),
                    input.as_vec_ref(),
                )?;

                // Validate receiver type
                let function_schema = definition
                    .interface
                    .functions
                    .get(ident)
                    .expect("Should exist due to schema check");
                match (&function_schema.receiver, node_id) {
                    (Some(receiver_info), Some(_)) => {
                        if is_direct_access
                            != receiver_info.ref_types.contains(RefTypes::DIRECT_ACCESS)
                        {
                            return Err(RuntimeError::SystemUpstreamError(
                                SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                            ));
                        }
                    }
                    (None, None) => {}
                    _ => {
                        return Err(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                        ));
                    }
                }

                // Execute
                let export = definition
                    .function_exports
                    .get(ident)
                    .expect("Schema should have validated this exists")
                    .clone();
                let output =
                    { V::invoke(&blueprint_id.package_address, export, input, &mut system)? };

                // Validate output
                system.validate_blueprint_payload(
                    &target,
                    BlueprintPayloadIdentifier::Function(ident.clone(), InputOrOutput::Output),
                    output.as_vec_ref(),
                )?;

                Ok(output)
            }
            Actor::BlueprintHook(BlueprintHookActor {
                blueprint_id, hook, ..
            }) => {
                // Find the export
                let definition = system.load_blueprint_definition(
                    blueprint_id.package_address,
                    &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                )?;
                let export =
                    definition
                        .hook_exports
                        .get(hook)
                        .ok_or(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::HookNotFound(hook.clone()),
                        ))?;

                // Input is not validated as they're created by system.

                // Invoke the export
                let output = V::invoke(
                    &blueprint_id.package_address,
                    export.clone(),
                    &input,
                    &mut system,
                )?;

                // Check output against well-known schema
                match hook {
                    BlueprintHook::OnVirtualize => {
                        scrypto_decode::<OnVirtualizeOutput>(output.as_slice()).map(|_| ())
                    }
                    BlueprintHook::OnDrop => {
                        scrypto_decode::<OnDropOutput>(output.as_slice()).map(|_| ())
                    }
                    BlueprintHook::OnMove => {
                        scrypto_decode::<OnMoveOutput>(output.as_slice()).map(|_| ())
                    }
                }
                .map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::OutputDecodeError(e))
                })?;

                Ok(output)
            }
        }
    }

    // Note: we check dangling nodes, in kernel, after auto-drop
    fn auto_drop<Y: KernelApi<CallbackObject = Self>>(
        nodes: Vec<NodeId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // Round 1 - drop all proofs
        for node_id in nodes {
            let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;

            match type_info {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_info: BlueprintInfo { blueprint_id, .. },
                    ..
                }) => {
                    match (
                        blueprint_id.package_address,
                        blueprint_id.blueprint_name.as_str(),
                    ) {
                        (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT) => {
                            let mut system = SystemService::new(api);
                            system.call_function(
                                RESOURCE_PACKAGE,
                                FUNGIBLE_PROOF_BLUEPRINT,
                                PROOF_DROP_IDENT,
                                scrypto_encode(&ProofDropInput {
                                    proof: Proof(Own(node_id)),
                                })
                                .unwrap(),
                            )?;
                        }
                        (RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                            let mut system = SystemService::new(api);
                            system.call_function(
                                RESOURCE_PACKAGE,
                                NON_FUNGIBLE_PROOF_BLUEPRINT,
                                PROOF_DROP_IDENT,
                                scrypto_encode(&ProofDropInput {
                                    proof: Proof(Own(node_id)),
                                })
                                .unwrap(),
                            )?;
                        }
                        _ => {
                            // no-op
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelInternalApi<System = Self>>(
        message: &CallFrameMessage,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_execution_finish(api, message)?;

        Ok(())
    }

    fn on_get_stack_id<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_get_stack_id(api)
    }

    fn on_switch_stack<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_switch_stack(api)
    }

    fn on_send_to_stack<Y: KernelInternalApi<System = Self>>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_send_to_stack(api, value.len())
    }

    fn on_set_call_frame_data<Y: KernelInternalApi<System = Self>>(
        data: &Self::CallFrameData,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_set_call_frame_data(api, data.len())
    }

    fn on_get_owned_nodes<Y: KernelInternalApi<System = Self>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_get_owned_nodes(api)
    }

    //--------------------------------------------------------------------------
    // Note that the following logic doesn't go through mixer and is not costed
    //--------------------------------------------------------------------------

    fn after_invoke<Y: KernelApi<CallbackObject = Self>>(
        output: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.kernel_get_system_state().current_call_frame;
        let is_to_barrier = current_actor.is_barrier();
        let destination_blueprint_id = current_actor.blueprint_id();
        for node_id in output.owned_nodes() {
            Self::on_move_node(
                node_id,
                false,
                is_to_barrier,
                destination_blueprint_id.clone(),
                api,
            )?;
        }

        SystemModuleMixer::after_invoke(api, output)
    }

    fn on_allocate_node_id<Y: KernelInternalApi<System = Self>>(
        entity_type: EntityType,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_allocate_node_id(api, entity_type)
    }

    fn on_mark_substate_as_transient<Y: KernelInternalApi<System = Self>>(
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_mark_substate_as_transient(
            api,
            node_id,
            partition_number,
            substate_key,
        )
    }

    fn on_substate_lock_fault<Y: KernelApi<CallbackObject = Self>>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        // As currently implemented, this should always be called with partition_num=0 and offset=0
        // since all nodes are access by accessing their type info first
        // This check is simply a sanity check that this invariant remain true
        if !partition_num.eq(&TYPE_INFO_FIELD_PARTITION)
            || !offset.eq(&TypeInfoField::TypeInfo.into())
        {
            return Ok(false);
        }

        let (blueprint_id, variant_id) = match node_id.entity_type() {
            Some(EntityType::GlobalPreallocatedSecp256k1Account) => (
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                ACCOUNT_CREATE_PREALLOCATED_SECP256K1_ID,
            ),
            Some(EntityType::GlobalPreallocatedEd25519Account) => (
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                ACCOUNT_CREATE_PREALLOCATED_ED25519_ID,
            ),
            Some(EntityType::GlobalPreallocatedSecp256k1Identity) => (
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                IDENTITY_CREATE_PREALLOCATED_SECP256K1_ID,
            ),
            Some(EntityType::GlobalPreallocatedEd25519Identity) => (
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                IDENTITY_CREATE_PREALLOCATED_ED25519_ID,
            ),
            _ => return Ok(false),
        };

        let mut service = SystemService::new(api);
        let definition = service.load_blueprint_definition(
            blueprint_id.package_address,
            &BlueprintVersionKey {
                blueprint: blueprint_id.blueprint_name.clone(),
                version: BlueprintVersion::default(),
            },
        )?;
        if definition
            .hook_exports
            .contains_key(&BlueprintHook::OnVirtualize)
        {
            let mut system = SystemService::new(api);
            let address = GlobalAddress::new_or_panic(node_id.into());
            let address_reservation =
                system.allocate_virtual_global_address(blueprint_id.clone(), address)?;

            api.kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::BlueprintHook(BlueprintHookActor {
                    blueprint_id: blueprint_id.clone(),
                    hook: BlueprintHook::OnVirtualize,
                    receiver: None,
                }),
                args: IndexedScryptoValue::from_typed(&OnVirtualizeInput {
                    variant_id,
                    rid: copy_u8_array(&node_id.as_bytes()[1..]),
                    address_reservation,
                }),
            }))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn on_drop_node_mut<Y: KernelApi<CallbackObject = Self>>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;

        match type_info {
            TypeInfoSubstate::Object(node_object_info) => {
                let mut service = SystemService::new(api);
                let definition = service.load_blueprint_definition(
                    node_object_info.blueprint_info.blueprint_id.package_address,
                    &BlueprintVersionKey {
                        blueprint: node_object_info
                            .blueprint_info
                            .blueprint_id
                            .blueprint_name
                            .clone(),
                        version: BlueprintVersion::default(),
                    },
                )?;
                if definition.hook_exports.contains_key(&BlueprintHook::OnDrop) {
                    api.kernel_invoke(Box::new(KernelInvocation {
                        call_frame_data: Actor::BlueprintHook(BlueprintHookActor {
                            blueprint_id: node_object_info.blueprint_info.blueprint_id.clone(),
                            hook: BlueprintHook::OnDrop,
                            receiver: Some(node_id.clone()),
                        }),
                        args: IndexedScryptoValue::from_typed(&OnDropInput {}),
                    }))
                    .map(|_| ())
                } else {
                    Ok(())
                }
            }
            TypeInfoSubstate::KeyValueStore(_)
            | TypeInfoSubstate::GlobalAddressReservation(_)
            | TypeInfoSubstate::GlobalAddressPhantom(_) => {
                // There is no way to drop a non-object through system API, triggering `NotAnObject` error.
                Ok(())
            }
        }
    }
}
