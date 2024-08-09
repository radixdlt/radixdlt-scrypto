use super::*;
use super::{FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::blueprints::package::PackageRoyaltyNativeBlueprint;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi, KernelInvocation};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, MoveModuleEvent,
    OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::object_modules::royalty::ComponentRoyaltyBlueprint;
use crate::system::actor::{Actor, FunctionActor, MethodActor, MethodType};
use crate::system::module::{InitSystemModule, SystemModule};
use crate::system::system_callback::System;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::{
    errors::{CanBeAbortion, RuntimeError, SystemModuleError},
    transaction::AbortReason,
};
use radix_engine_interface::api::AttachedModuleId;
use radix_engine_interface::blueprints::package::BlueprintVersionKey;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::types::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
}

impl CanBeAbortion for CostingError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::FeeReserveError(err) => err.abortion(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum OnApplyCost {
    #[default]
    Normal,
    ForceFailOnCount {
        fail_after: Rc<RefCell<u64>>,
    },
}

impl OnApplyCost {
    pub fn on_call(&mut self) -> Result<(), CostingError> {
        match self {
            OnApplyCost::Normal => {}
            OnApplyCost::ForceFailOnCount { fail_after } => {
                if *fail_after.borrow() == 0 {
                    return Ok(());
                }

                *fail_after.borrow_mut() -= 1;
                if *fail_after.borrow() == 0 {
                    return Err(CostingError::FeeReserveError(
                        FeeReserveError::InsufficientBalance {
                            required: Decimal::MAX,
                            remaining: Decimal::ONE,
                        },
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CostingModuleConfig {
    /// The maximum allowed method royalty in XRD allowed to be set by package and component owners
    pub max_per_function_royalty_in_xrd: Decimal,
    /// If true, execution costing for all system calls will occur
    pub apply_execution_cost_2: bool,
    /// If true, costing on reference checks on boot will occur
    pub apply_boot_ref_check_costing: bool,
}

impl CostingModuleConfig {
    pub fn babylon_genesis() -> Self {
        Self {
            max_per_function_royalty_in_xrd: Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD)
                .unwrap(),
            apply_execution_cost_2: false,
            apply_boot_ref_check_costing: false,
        }
    }

    pub fn bottlenose() -> Self {
        Self {
            max_per_function_royalty_in_xrd: Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD)
                .unwrap(),
            apply_execution_cost_2: true,
            apply_boot_ref_check_costing: true,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct DetailedExecutionCostBreakdownEntry {
    pub depth: usize,
    pub item: ExecutionCostBreakdownItem,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum ExecutionCostBreakdownItem {
    Invocation {
        actor: Actor,
        args: (ScryptoOwnedRawValue,),
    },
    InvocationComplete,
    Execution {
        simple_name: String,
        item: owned::ExecutionCostingEntryOwned,
        cost_units: u32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct CostBreakdown {
    pub execution_cost_breakdown: IndexMap<String, u32>,
    pub finalization_cost_breakdown: IndexMap<String, u32>,
    pub storage_cost_breakdown: IndexMap<StorageType, usize>,
}

#[derive(Debug, Clone, Default)]
pub struct DetailedCostBreakdown {
    /// A more detailed cost breakdown with information on the depth.
    pub detailed_execution_cost_breakdown: Vec<DetailedExecutionCostBreakdownEntry>,
}

#[derive(Debug, Clone)]
pub struct CostingModule {
    pub config: CostingModuleConfig,
    pub fee_reserve: SystemLoanFeeReserve,
    pub fee_table: FeeTable,
    pub on_apply_cost: OnApplyCost,

    pub tx_payload_len: usize,
    pub tx_num_of_signature_validations: usize,
    pub cost_breakdown: Option<CostBreakdown>,
    pub detailed_cost_breakdown: Option<DetailedCostBreakdown>,

    /// This keeps track of the current kernel depth.
    pub current_depth: usize,
}

impl CostingModule {
    pub fn apply_execution_cost(
        &mut self,
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), CostingError> {
        self.on_apply_cost.on_call()?;

        let cost_units = costing_entry.to_execution_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_execution(cost_units)
            .map_err(CostingError::FeeReserveError)?;

        if let Some(cost_breakdown) = &mut self.cost_breakdown {
            let key = costing_entry.to_trace_key();
            cost_breakdown
                .execution_cost_breakdown
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }
        if let Some(detailed_cost_breakdown) = &mut self.detailed_cost_breakdown {
            // Add an entry for the more detailed execution cost
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth: self.current_depth,
                    item: ExecutionCostBreakdownItem::Execution {
                        simple_name: costing_entry.to_trace_key(),
                        item: owned::ExecutionCostingEntryOwned::from(costing_entry),
                        cost_units,
                    },
                });
        }

        Ok(())
    }

    pub fn apply_execution_cost_2(
        &mut self,
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), CostingError> {
        if self.config.apply_execution_cost_2 {
            self.apply_execution_cost(costing_entry)
        } else {
            Ok(())
        }
    }

    pub fn apply_deferred_execution_cost(
        &mut self,
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), CostingError> {
        if let ExecutionCostingEntry::RefCheck { .. } = &costing_entry {
            if !self.config.apply_boot_ref_check_costing {
                return Ok(());
            }
        }

        self.on_apply_cost.on_call()?;

        let cost_units = costing_entry.to_execution_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_deferred_execution(cost_units)
            .map_err(CostingError::FeeReserveError)?;

        if let Some(cost_breakdown) = &mut self.cost_breakdown {
            let key = costing_entry.to_trace_key();
            cost_breakdown
                .execution_cost_breakdown
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }
        if let Some(detailed_cost_breakdown) = &mut self.detailed_cost_breakdown {
            // Add an entry for the more detailed execution cost
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth: 0,
                    item: ExecutionCostBreakdownItem::Execution {
                        simple_name: costing_entry.to_trace_key(),
                        item: owned::ExecutionCostingEntryOwned::from(costing_entry),
                        cost_units,
                    },
                });
        }

        Ok(())
    }

    pub fn apply_deferred_storage_cost(
        &mut self,
        storage_type: StorageType,
        size_increase: usize,
    ) -> Result<(), CostingError> {
        self.on_apply_cost.on_call()?;

        self.fee_reserve
            .consume_deferred_storage(storage_type, size_increase)
            .map_err(CostingError::FeeReserveError)?;

        if let Some(cost_breakdown) = &mut self.cost_breakdown {
            cost_breakdown
                .storage_cost_breakdown
                .entry(storage_type)
                .or_default()
                .add_assign(size_increase);
        }

        Ok(())
    }

    pub fn apply_finalization_cost(
        &mut self,
        costing_entry: FinalizationCostingEntry,
    ) -> Result<(), CostingError> {
        self.on_apply_cost.on_call()?;

        let cost_units = costing_entry.to_finalization_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_finalization(cost_units)
            .map_err(CostingError::FeeReserveError)?;

        if let Some(cost_breakdown) = &mut self.cost_breakdown {
            let key = costing_entry.to_trace_key();
            cost_breakdown
                .finalization_cost_breakdown
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }

        Ok(())
    }

    pub fn apply_storage_cost(
        &mut self,
        storage_type: StorageType,
        size_increase: usize,
    ) -> Result<(), CostingError> {
        self.on_apply_cost.on_call()?;

        self.fee_reserve
            .consume_storage(storage_type, size_increase)
            .map_err(CostingError::FeeReserveError)?;

        if let Some(cost_breakdown) = &mut self.cost_breakdown {
            cost_breakdown
                .storage_cost_breakdown
                .entry(storage_type)
                .or_default()
                .add_assign(size_increase);
        }

        Ok(())
    }

    pub fn lock_fee(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) {
        self.fee_reserve.lock_fee(vault_id, locked_fee, contingent);
    }
}

pub fn apply_royalty_cost<Y: KernelApi<System<V>>, V: SystemCallbackObject>(
    api: &mut Y,
    royalty_amount: RoyaltyAmount,
    recipient: RoyaltyRecipient,
) -> Result<(), RuntimeError> {
    api.kernel_get_system()
        .modules
        .costing
        .on_apply_cost
        .on_call()
        .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

    api.kernel_get_system()
        .modules
        .costing
        .fee_reserve
        .consume_royalty(royalty_amount, recipient)
        .map_err(|e| {
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(e),
            ))
        })
}

impl InitSystemModule for CostingModule {
    fn init(&mut self) -> Result<(), BootloadingError> {
        self.apply_deferred_execution_cost(ExecutionCostingEntry::ValidateTxPayload {
            size: self.tx_payload_len,
        })
        .map_err(BootloadingError::FailedToApplyDeferredCosts)?;

        self.apply_deferred_execution_cost(ExecutionCostingEntry::VerifyTxSignatures {
            num_signatures: self.tx_num_of_signature_validations,
        })
        .map_err(BootloadingError::FailedToApplyDeferredCosts)?;

        self.apply_deferred_storage_cost(StorageType::Archive, self.tx_payload_len)
            .map_err(BootloadingError::FailedToApplyDeferredCosts)?;

        Ok(())
    }
}

impl<V: SystemCallbackObject> SystemModule<System<V>> for CostingModule {
    fn before_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();

        // Add invocation information to the execution cost breakdown.
        if let Some(ref mut detailed_cost_breakdown) = api
            .kernel_get_system()
            .modules
            .costing
            .detailed_cost_breakdown
        {
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth,
                    item: ExecutionCostBreakdownItem::Invocation {
                        actor: invocation.call_frame_data.clone(),
                        args: (invocation.args.value().ref_into_owned(),),
                    },
                });
        }

        // Skip invocation costing for transaction processor
        if depth == 0 {
            return Ok(());
        }

        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::BeforeInvoke {
                actor: &invocation.call_frame_data,
                input_size: invocation.len(),
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        // Identify the function, and optional component address
        let (optional_blueprint_id, ident, maybe_object_royalties) = {
            let (maybe_component, ident) = match &invocation.call_frame_data {
                Actor::Method(MethodActor {
                    method_type,
                    node_id,
                    ident,
                    object_info,
                    ..
                }) => {
                    // Only do royalty costing for Main
                    match method_type {
                        MethodType::Main | MethodType::Direct => {}
                        MethodType::Module(..) => return Ok(()),
                    }

                    match &object_info.object_type {
                        ObjectType::Global { modules }
                            if modules.contains_key(&AttachedModuleId::Royalty) =>
                        {
                            (Some(*node_id), ident)
                        }
                        _ => (None, ident),
                    }
                }
                Actor::Function(FunctionActor { ident, .. }) => (None, ident),
                Actor::BlueprintHook(..) | Actor::Root => {
                    return Ok(());
                }
            };

            (
                invocation.call_frame_data.blueprint_id(),
                ident,
                maybe_component,
            )
        };

        //===========================
        // Apply package royalty
        //===========================
        if let Some(blueprint_id) = optional_blueprint_id {
            let bp_version_key =
                BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str());
            PackageRoyaltyNativeBlueprint::charge_package_royalty(
                blueprint_id.package_address.as_node_id(),
                &bp_version_key,
                ident,
                api,
            )?;
        }

        //===========================
        // Apply component royalty
        //===========================
        if let Some(node_id) = maybe_object_royalties {
            ComponentRoyaltyBlueprint::charge_component_royalty(&node_id, ident, api)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn after_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        output: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();

        // Add invocation information to the execution cost breakdown.
        if let Some(ref mut detailed_cost_breakdown) = api
            .kernel_get_system()
            .modules
            .costing
            .detailed_cost_breakdown
        {
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth,
                    item: ExecutionCostBreakdownItem::InvocationComplete,
                });
        }

        // Skip invocation costing for transaction processor
        if depth == 0 {
            return Ok(());
        }

        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::AfterInvoke {
                output_size: output.payload_len(),
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_create_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::CreateNode { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_pin_node(system: &mut System<V>, node_id: &NodeId) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::PinNode { node_id })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_drop_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::DropNode { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_move_module<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &MoveModuleEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::MoveModule { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_open_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::OpenSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_mark_substate_as_transient(
        system: &mut System<V>,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::MarkSubstateAsTransient {
                node_id,
                partition_number,
                substate_key,
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_read_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::ReadSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_write_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::WriteSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_close_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::CloseSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_set_substate(
        system: &mut System<V>,
        event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::SetSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_remove_substate(
        system: &mut System<V>,
        event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::RemoveSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_scan_keys(system: &mut System<V>, event: &ScanKeysEvent) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::ScanKeys { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_drain_substates(
        system: &mut System<V>,
        event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::DrainSubstates { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_scan_sorted_substates(
        system: &mut System<V>,
        event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::ScanSortedSubstates { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<System<V>>>(
        api: &mut Y,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let costing_module = &mut api.kernel_get_system().modules.costing;
        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::AllocateNodeId)
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }
}
