use super::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::blueprints::account::ACCOUNT_CREATE_VIRTUAL_ED25519_ID;
use crate::blueprints::account::ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID;
use crate::blueprints::identity::IDENTITY_CREATE_VIRTUAL_ED25519_ID;
use crate::blueprints::identity::IDENTITY_CREATE_VIRTUAL_SECP256K1_ID;
use crate::blueprints::transaction_processor::TransactionProcessorRunInputEfficientEncodable;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::kernel::kernel_api::{KernelInternalApi, KernelSubstateApi};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::system::actor::Actor;
use crate::system::actor::BlueprintHookActor;
use crate::system::actor::FunctionActor;
use crate::system::actor::MethodActor;
use crate::system::module::{InitSystemModule, SystemModule};
use crate::system::system::SystemService;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::AuthModule;
use crate::system::system_modules::costing::{CostingModule, FeeTable, SystemLoanFeeReserve};
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::kernel_trace::KernelTraceModule;
use crate::system::system_modules::limits::LimitsModule;
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_modules::{EnabledModules, SystemModuleMixer};
use crate::system::system_substates::KeyValueEntrySubstate;
use crate::system::system_type_checker::{BlueprintTypeTarget, KVStoreTypeTarget};
use crate::track::{BootStore, CommitableSubstateStore, Track};
use crate::transaction::{CostingParameters, LimitParameters, SystemOverrides};
use radix_blueprint_schema_init::RefTypes;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::api::{ClientBlueprintApi, CollectionIndex};
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::hooks::OnDropInput;
use radix_engine_interface::blueprints::hooks::OnDropOutput;
use radix_engine_interface::blueprints::hooks::OnMoveInput;
use radix_engine_interface::blueprints::hooks::OnMoveOutput;
use radix_engine_interface::blueprints::hooks::OnVirtualizeInput;
use radix_engine_interface::blueprints::hooks::OnVirtualizeOutput;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_substate_store_interface::{db_key_mapper::SpreadPrefixKeyMapper, interface::*};
use radix_transactions::model::{Executable, PreAllocatedAddress};
use crate::blueprints::consensus_manager::{ConsensusManagerField, ConsensusManagerStateFieldPayload};
use crate::blueprints::transaction_tracker::{TransactionStatus, TransactionStatusV1, TransactionTrackerSubstate};

pub const BOOT_LOADER_SYSTEM_SUBSTATE_FIELD_KEY: FieldKey = 1u8;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct SystemParameters {
    pub network_definition: NetworkDefinition,
    pub costing_parameters: CostingParameters,
    pub limit_parameters: LimitParameters,
    pub max_per_function_royalty_in_xrd: Decimal,
}

impl Default for SystemParameters {
    fn default() -> Self {
        Self {
            network_definition: NetworkDefinition::mainnet(),
            costing_parameters: CostingParameters::default(),
            limit_parameters: LimitParameters::default(),
            max_per_function_royalty_in_xrd: Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD)
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SystemBoot {
    V1(SystemParameters),
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

#[derive(Clone)]
pub struct SystemInit<C> {
    // These fields only affect side effects and do not affect ledger state execution
    pub enable_kernel_trace: bool,
    pub enable_cost_breakdown: bool,
    pub execution_trace: Option<usize>,

    // Higher layer initialization object
    pub callback_init: C,

    // An override of system configuration
    pub system_overrides: Option<SystemOverrides>,
}

pub struct System<C: SystemCallbackObject> {
    pub callback: C,
    pub blueprint_cache: NonIterMap<CanonicalBlueprintId, Rc<BlueprintDefinition>>,
    pub schema_cache: NonIterMap<SchemaHash, Rc<VersionedScryptoSchema>>,
    pub auth_cache: NonIterMap<CanonicalBlueprintId, AuthConfig>,
    pub modules: SystemModuleMixer,
}

impl<C: SystemCallbackObject> System<C> {
    pub fn read_epoch<S: SubstateDatabase>(track: &mut Track<S, SpreadPrefixKeyMapper>) -> Option<Epoch> {
        // TODO - Instead of doing a check of the exact epoch, we could do a check in range [X, Y]
        //        Which could allow for better caching of transaction validity over epoch boundaries
        match track.read_substate(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::State.into(),
        ) {
            Some(x) => {
                let substate: FieldSubstate<ConsensusManagerStateFieldPayload> =
                    x.as_typed().unwrap();
                Some(substate.into_payload().into_latest().epoch)
            }
            None => None,
        }
    }

    pub fn validate_epoch_range(
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
                valid_until: end_epoch_exclusive.previous(),
                current_epoch,
            });
        }

        Ok(())
    }

    pub fn validate_intent_hash<S: SubstateDatabase>(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        intent_hash: Hash,
        expiry_epoch: Epoch,
    ) -> Result<(), RejectionReason> {
        let substate: FieldSubstate<TransactionTrackerSubstate> = track
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

        let substate = track.read_substate(
            TRANSACTION_TRACKER.as_node_id(),
            PartitionNumber(partition_number),
            &SubstateKey::Map(scrypto_encode(&intent_hash).unwrap()),
        );

        match substate {
            Some(value) => {
                let substate: KeyValueEntrySubstate<TransactionStatus> = value.as_typed().unwrap();
                match substate.into_value() {
                    Some(status) => match status.into_v1() {
                        TransactionStatusV1::CommittedSuccess
                        | TransactionStatusV1::CommittedFailure => {
                            return Err(RejectionReason::IntentHashPreviouslyCommitted);
                        }
                        TransactionStatusV1::Cancelled => {
                            return Err(RejectionReason::IntentHashPreviouslyCancelled);
                        }
                    },
                    None => {}
                }
            }
            None => {}
        }

        Ok(())
    }
}

impl<C: SystemCallbackObject> KernelCallbackObject for System<C> {
    type LockData = SystemLockData;
    type CallFrameData = Actor;
    type InitInput = SystemInit<C::InitInput>;

    fn init<S: BootStore>(
        store: &S,
        executable: &Executable,
        init_input: SystemInit<C::InitInput>,
    ) -> Result<Self, BootloadingError> {
        let mut system_parameters = {
            let system_boot = store
                .read_boot_substate(
                    TRANSACTION_TRACKER.as_node_id(),
                    BOOT_LOADER_PARTITION,
                    &SubstateKey::Field(BOOT_LOADER_SYSTEM_SUBSTATE_FIELD_KEY),
                )
                .map(|v| scrypto_decode(v.as_slice()).unwrap())
                .unwrap_or(SystemBoot::V1(SystemParameters::default()));

            match system_boot {
                SystemBoot::V1(system_parameters) => system_parameters,
            }
        };

        let callback = C::init(store, init_input.callback_init)?;

        let enabled_modules = {
            let mut enabled_modules = EnabledModules::AUTH | EnabledModules::TRANSACTION_RUNTIME;
            if !executable.is_system() {
                enabled_modules |= EnabledModules::LIMITS;
                enabled_modules |= EnabledModules::COSTING;
            };

            if init_input.enable_kernel_trace {
                enabled_modules |= EnabledModules::KERNEL_TRACE;
            }
            if init_input.execution_trace.is_some() {
                enabled_modules |= EnabledModules::EXECUTION_TRACE;
            }

            if let Some(system_overrides) = &init_input.system_overrides {
                if system_overrides.disable_auth {
                    enabled_modules &= !EnabledModules::AUTH;
                }
                if system_overrides.disable_costing {
                    enabled_modules &= !EnabledModules::COSTING;
                }
                if system_overrides.disable_limits {
                    enabled_modules &= !EnabledModules::LIMITS;
                }
            }

            enabled_modules
        };

        if let Some(system_overrides) = init_input.system_overrides {
            if let Some(costing_override) = system_overrides.costing_parameters {
                system_parameters.costing_parameters = costing_override;
            }

            if let Some(limits_override) = system_overrides.limit_parameters {
                system_parameters.limit_parameters = limits_override;
            }

            if let Some(network_definition) = system_overrides.network_definition {
                system_parameters.network_definition = network_definition;
            }
        }

        let txn_runtime_module = TransactionRuntimeModule::new(
            system_parameters.network_definition,
            executable.intent_hash().to_hash(),
        );

        let auth_module = AuthModule::new(executable.auth_zone_params().clone());
        let limits_module = { LimitsModule::from_params(system_parameters.limit_parameters) };

        let costing_module = CostingModule {
            fee_reserve: SystemLoanFeeReserve::new(
                &system_parameters.costing_parameters,
                executable.costing_parameters(),
            ),
            fee_table: FeeTable::new(),
            tx_payload_len: executable.payload_size(),
            tx_num_of_signature_validations: executable.num_of_signature_validations(),
            max_per_function_royalty_in_xrd: system_parameters.max_per_function_royalty_in_xrd,
            cost_breakdown: if init_input.enable_cost_breakdown {
                Some(Default::default())
            } else {
                None
            },
            on_apply_cost: Default::default(),
        };

        let mut modules = SystemModuleMixer::new(
            enabled_modules,
            KernelTraceModule,
            txn_runtime_module,
            auth_module,
            limits_module,
            costing_module,
            ExecutionTraceModule::new(
                init_input
                    .execution_trace
                    .unwrap_or(MAX_EXECUTION_TRACE_DEPTH),
            ),
        );

        modules.init()?;

        Ok(System {
            blueprint_cache: NonIterMap::new(),
            auth_cache: NonIterMap::new(),
            schema_cache: NonIterMap::new(),
            callback,
            modules,
        })
    }

    fn init2<S: SubstateDatabase>(&self, track: &mut Track<S, SpreadPrefixKeyMapper>, executable: &Executable) -> Result<(), RejectionReason> {
        // Perform runtime validation.
        // TODO: the following assumptions can be removed with better interface.
        // We are assuming that intent hash store is ready when epoch manager is ready.
        let current_epoch = Self::read_epoch(track);
        if let Some(current_epoch) = current_epoch {
            if let Some(range) = executable.epoch_range() {
                Self::validate_epoch_range(
                    current_epoch,
                    range.start_epoch_inclusive,
                    range.end_epoch_exclusive,
                )
                    .and_then(|_| {
                        Self::validate_intent_hash(
                            track,
                            executable.intent_hash().to_hash(),
                            range.end_epoch_exclusive,
                        )
                    })
                    .and_then(|_| Ok(()))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn start<Y>(
        api: &mut Y,
        manifest_encoded_instructions: &[u8],
        pre_allocated_addresses: &Vec<PreAllocatedAddress>,
        references: &IndexSet<Reference>,
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let mut system = SystemService::new(api);

        // Allocate global addresses
        let mut global_address_reservations = Vec::new();
        for PreAllocatedAddress {
            blueprint_id,
            address,
        } in pre_allocated_addresses
        {
            let global_address_reservation =
                system.prepare_global_address(blueprint_id.clone(), address.clone())?;
            global_address_reservations.push(global_address_reservation);
        }

        // Call TX processor
        let rtn = system.call_function(
            TRANSACTION_PROCESSOR_PACKAGE,
            TRANSACTION_PROCESSOR_BLUEPRINT,
            TRANSACTION_PROCESSOR_RUN_IDENT,
            scrypto_encode(&TransactionProcessorRunInputEfficientEncodable {
                manifest_encoded_instructions,
                global_address_reservations,
                references,
                blobs,
            })
            .unwrap(),
        )?;

        Ok(rtn)
    }

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_teardown(api)
    }

    fn on_pin_node(&mut self, node_id: &NodeId) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_pin_node(self, node_id)
    }

    fn on_create_node<Y>(api: &mut Y, event: CreateNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_create_node(api, &event)
    }

    fn on_drop_node<Y>(api: &mut Y, event: DropNodeEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_drop_node(api, &event)
    }

    fn on_move_module<Y>(api: &mut Y, event: MoveModuleEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_move_module(api, &event)
    }

    fn on_open_substate<Y>(api: &mut Y, event: OpenSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_open_substate(api, &event)
    }

    fn on_close_substate<Y>(api: &mut Y, event: CloseSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_close_substate(api, &event)
    }

    fn on_read_substate<Y>(api: &mut Y, event: ReadSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_read_substate(api, &event)
    }

    fn on_write_substate<Y>(api: &mut Y, event: WriteSubstateEvent) -> Result<(), RuntimeError>
    where
        Y: KernelInternalApi<Self>,
    {
        SystemModuleMixer::on_write_substate(api, &event)
    }

    fn on_set_substate(&mut self, event: SetSubstateEvent) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_set_substate(self, &event)
    }

    fn on_remove_substate(&mut self, event: RemoveSubstateEvent) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_remove_substate(self, &event)
    }

    fn on_scan_keys(&mut self, event: ScanKeysEvent) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_scan_keys(self, &event)
    }

    fn on_drain_substates(&mut self, event: DrainSubstatesEvent) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_drain_substates(self, &event)
    }

    fn on_scan_sorted_substates(
        &mut self,
        event: ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_scan_sorted_substates(self, &event)
    }

    fn before_invoke<Y>(
        invocation: &KernelInvocation<Actor>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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

    fn after_invoke<Y>(output: &IndexedScryptoValue, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_start(api)
    }

    fn on_execution_finish<Y>(message: &CallFrameMessage, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, message)?;

        Ok(())
    }

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_allocate_node_id(api, entity_type)
    }

    //--------------------------------------------------------------------------
    // Note that the following logic doesn't go through mixer and is not costed
    //--------------------------------------------------------------------------

    fn invoke_upstream<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<System<C>>,
    {
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
                    { C::invoke(&blueprint_id.package_address, export, input, &mut system)? };

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
                let output = C::invoke(
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
    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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

    fn on_mark_substate_as_transient(
        &mut self,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        SystemModuleMixer::on_mark_substate_as_transient(
            self,
            node_id,
            partition_number,
            substate_key,
        )
    }

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        // As currently implemented, this should always be called with partition_num=0 and offset=0
        // since all nodes are access by accessing their type info first
        // This check is simply a sanity check that this invariant remain true
        if !partition_num.eq(&TYPE_INFO_FIELD_PARTITION)
            || !offset.eq(&TypeInfoField::TypeInfo.into())
        {
            return Ok(false);
        }

        let (blueprint_id, variant_id) = match node_id.entity_type() {
            Some(EntityType::GlobalVirtualSecp256k1Account) => (
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID,
            ),
            Some(EntityType::GlobalVirtualEd25519Account) => (
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                ACCOUNT_CREATE_VIRTUAL_ED25519_ID,
            ),
            Some(EntityType::GlobalVirtualSecp256k1Identity) => (
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                IDENTITY_CREATE_VIRTUAL_SECP256K1_ID,
            ),
            Some(EntityType::GlobalVirtualEd25519Identity) => (
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                IDENTITY_CREATE_VIRTUAL_ED25519_ID,
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

    fn on_drop_node_mut<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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

    fn on_move_node<Y>(
        node_id: &NodeId,
        is_moving_down: bool,
        is_to_barrier: bool,
        destination_blueprint_id: Option<BlueprintId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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
