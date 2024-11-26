use super::*;
use super::{FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::blueprints::package::PackageRoyaltyNativeBlueprint;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::object_modules::royalty::ComponentRoyaltyBlueprint;
use crate::system::actor::{Actor, FunctionActor, MethodActor, MethodType};
use crate::system::module::*;
use crate::system::system_callback::*;
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
        args: (ScryptoValue,),
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
        if let ExecutionCostingEntry::CheckReference { .. } = &costing_entry {
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

    pub fn apply_deferred_finalization_cost(
        &mut self,
        costing_entry: FinalizationCostingEntry,
    ) -> Result<(), CostingError> {
        self.on_apply_cost.on_call()?;

        let cost_units = costing_entry.to_finalization_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_deferred_finalization(cost_units)
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

    pub fn unpack_for_receipt(
        self,
    ) -> (
        SystemLoanFeeReserve,
        Option<CostBreakdown>,
        Option<DetailedCostBreakdown>,
    ) {
        (
            self.fee_reserve,
            self.cost_breakdown,
            self.detailed_cost_breakdown,
        )
    }
}

pub fn apply_royalty_cost(
    api: &mut impl SystemModuleApiFor<CostingModule>,
    royalty_amount: RoyaltyAmount,
    recipient: RoyaltyRecipient,
) -> Result<(), RuntimeError> {
    api.module()
        .on_apply_cost
        .on_call()
        .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

    api.module()
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

impl ResolvableSystemModule for CostingModule {
    #[inline]
    fn resolve_from_system(system: &mut impl HasModules) -> &mut Self {
        &mut system.modules_mut().costing
    }
}

impl PrivilegedSystemModule for CostingModule {
    /// Runs after SystemModule::before_invoke
    /// Called from the Module Mixer
    fn privileged_before_invoke(
        api: &mut impl SystemBasedKernelApi,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        // This check only applies in SystemV1.
        // In this case, there was a call from Root => Transaction Processor, which this check avoids charging for.
        // From SystemV2 onwards, there is no explicit call, and the Root actor is simply overwritten.
        if api.kernel_get_system_state().current_call_frame.is_root() {
            return Ok(());
        }

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
}

impl<ModuleApi: SystemModuleApiFor<Self>> SystemModule<ModuleApi> for CostingModule {
    fn before_invoke(
        api: &mut ModuleApi,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        let depth = api.current_stack_depth_uncosted();
        let is_root = api.system_state().current_call_frame.is_root();
        let costing_module = api.module();

        // Add invocation information to the execution cost breakdown.
        if let Some(ref mut detailed_cost_breakdown) = costing_module.detailed_cost_breakdown {
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth,
                    item: ExecutionCostBreakdownItem::Invocation {
                        actor: invocation.call_frame_data.clone(),
                        args: (invocation.args.as_scrypto_value().to_owned(),),
                    },
                });
        }

        // This check only applies in SystemV1.
        // In this case, there was a call from Root => Transaction Processor, which this check avoids charging for.
        // From SystemV2 onwards, there is no explicit call, and the Root actor is simply overwritten.
        if is_root {
            return Ok(());
        }

        costing_module.current_depth = depth;
        costing_module
            .apply_execution_cost(ExecutionCostingEntry::BeforeInvoke {
                actor: &invocation.call_frame_data,
                input_size: invocation.len(),
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        // NOTE: privileged_before_invoke is now called by the module mixer

        Ok(())
    }

    #[inline(always)]
    fn after_invoke(api: &mut ModuleApi, output: &IndexedScryptoValue) -> Result<(), RuntimeError> {
        let depth = api.current_stack_depth_uncosted();

        // Add invocation information to the execution cost breakdown.
        if let Some(ref mut detailed_cost_breakdown) = api.module().detailed_cost_breakdown {
            detailed_cost_breakdown
                .detailed_execution_cost_breakdown
                .push(DetailedExecutionCostBreakdownEntry {
                    depth,
                    item: ExecutionCostBreakdownItem::InvocationComplete,
                });
        }

        // This check only applies in SystemV1.
        // In this case, there was a call from Root => Transaction Processor, which this check avoids charging for.
        // From SystemV2 onwards, there is no explicit call, and the Root actor is simply overwritten.
        if api.system_state().current_call_frame.is_root() {
            return Ok(());
        }

        api.module().current_depth = depth;
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::AfterInvoke {
                output_size: output.len(),
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_create_node(api: &mut ModuleApi, event: &CreateNodeEvent) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::CreateNode { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_pin_node(api: &mut ModuleApi, node_id: &NodeId) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::PinNode { node_id })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_drop_node(api: &mut ModuleApi, event: &DropNodeEvent) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::DropNode { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_move_module(api: &mut ModuleApi, event: &MoveModuleEvent) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::MoveModule { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_open_substate(
        api: &mut ModuleApi,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::OpenSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_mark_substate_as_transient(
        api: &mut ModuleApi,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::MarkSubstateAsTransient {
                node_id,
                partition_number,
                substate_key,
            })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_read_substate(
        api: &mut ModuleApi,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::ReadSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_write_substate(
        api: &mut ModuleApi,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::WriteSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_close_substate(
        api: &mut ModuleApi,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::CloseSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_set_substate(api: &mut ModuleApi, event: &SetSubstateEvent) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::SetSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_remove_substate(
        api: &mut ModuleApi,
        event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::RemoveSubstate { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_scan_keys(api: &mut ModuleApi, event: &ScanKeysEvent) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::ScanKeys { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_drain_substates(
        api: &mut ModuleApi,
        event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::DrainSubstates { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_scan_sorted_substates(
        api: &mut ModuleApi,
        event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::ScanSortedSubstates { event })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_allocate_node_id(
        api: &mut ModuleApi,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        api.module().current_depth = api.current_stack_depth_uncosted();
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::AllocateNodeId)
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_get_stack_id(api: &mut ModuleApi) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::GetStackId)
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_switch_stack(api: &mut ModuleApi) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::SwitchStack)
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_send_to_stack(api: &mut ModuleApi, data_len: usize) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::SendToStack { data_len })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_set_call_frame_data(api: &mut ModuleApi, data_len: usize) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::SetCallFrameData { data_len })
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }

    fn on_get_owned_nodes(api: &mut ModuleApi) -> Result<(), RuntimeError> {
        api.module()
            .apply_execution_cost(ExecutionCostingEntry::GetOwnedNodes)
            .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;

        Ok(())
    }
}
