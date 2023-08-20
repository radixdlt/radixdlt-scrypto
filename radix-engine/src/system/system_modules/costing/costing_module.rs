use super::*;
use super::{FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::blueprints::package::PackageRoyaltyNativeBlueprint;
use crate::kernel::actor::{Actor, FunctionActor, MethodActor};
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi, KernelInvocation};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, MoveModuleEvent,
    OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::system::module::SystemModule;
use crate::system::node_modules::royalty::ComponentRoyaltyBlueprint;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::{
    errors::{CanBeAbortion, RuntimeError, SystemModuleError},
    transaction::AbortReason,
};
use radix_engine_interface::api::{ModuleId, ObjectModuleId};
use radix_engine_interface::blueprints::package::BlueprintVersionKey;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::{types::NodeId, *};

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

#[derive(Debug, Clone)]
pub struct CostingModule {
    pub fee_reserve: SystemLoanFeeReserve,
    pub fee_table: FeeTable,
    pub max_call_depth: usize,
    pub tx_payload_len: usize,
    pub tx_signature_size: usize,
    /// The maximum allowed method royalty in XRD allowed to be set by package and component owners
    pub max_per_function_royalty_in_xrd: Decimal,
    pub enable_cost_breakdown: bool,
    pub execution_cost_breakdown: IndexMap<String, u32>,
    pub finalization_cost_breakdown: IndexMap<String, u32>,
}

impl CostingModule {
    pub fn fee_reserve(self) -> SystemLoanFeeReserve {
        self.fee_reserve
    }

    pub fn apply_execution_cost(
        &mut self,
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), RuntimeError> {
        let cost_units = costing_entry.to_execution_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_execution(cost_units)
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })?;

        if self.enable_cost_breakdown {
            let key = costing_entry.to_trace_key();
            self.execution_cost_breakdown
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }

        Ok(())
    }

    pub fn apply_deferred_execution_cost(
        &mut self,
        costing_entry: ExecutionCostingEntry,
    ) -> Result<(), RuntimeError> {
        let cost_units = costing_entry.to_execution_cost_units(&self.fee_table);

        self.fee_reserve.consume_deferred(cost_units).map_err(|e| {
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(e),
            ))
        })?;

        if self.enable_cost_breakdown {
            let key = costing_entry.to_trace_key();
            self.execution_cost_breakdown
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }

        Ok(())
    }

    pub fn apply_finalization_cost(
        &mut self,
        costing_entry: FinalizationCostingEntry,
    ) -> Result<(), RuntimeError> {
        let cost_units = costing_entry.to_finalization_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_finalization(cost_units)
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })?;

        if self.enable_cost_breakdown {
            let key = costing_entry.to_trace_key();
            self.finalization_cost_breakdown
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
    ) -> Result<(), RuntimeError> {
        self.fee_reserve
            .consume_storage(storage_type, size_increase)
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })?;

        Ok(())
    }

    pub fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        self.fee_reserve
            .lock_fee(vault_id, locked_fee, contingent)
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })
    }
}

pub fn apply_royalty_cost<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
    api: &mut Y,
    royalty_amount: RoyaltyAmount,
    recipient: RoyaltyRecipient,
) -> Result<(), RuntimeError> {
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

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for CostingModule {
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let costing = &mut api.kernel_get_system().modules.costing;

        costing.apply_deferred_execution_cost(ExecutionCostingEntry::ValidateTxPayload {
            size: costing.tx_payload_len,
        })?;
        costing.apply_deferred_execution_cost(ExecutionCostingEntry::VerifyTxSignatures {
            num_signatures: costing.tx_signature_size,
        })?;

        Ok(())
    }

    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        // Skip invocation costing for transaction processor
        if api.kernel_get_current_depth() == 0 {
            return Ok(());
        }

        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::BeforeInvoke {
                actor: &invocation.call_frame_data,
                input_size: invocation.len(),
            })?;

        // Identify the function, and optional component address
        let (optional_blueprint_id, ident, maybe_object_royalties) = {
            let (maybe_component, ident) = match &invocation.call_frame_data {
                Actor::Method(MethodActor {
                    node_id,
                    module_id,
                    ident,
                    object_info,
                    ..
                }) => {
                    // Only do royalty costing for Main
                    if module_id.ne(&ObjectModuleId::Main) {
                        return Ok(());
                    }

                    if object_info
                        .module_versions
                        .contains_key(&ModuleId::Royalty)
                    {
                        (Some(node_id.clone()), ident)
                    } else {
                        (None, ident)
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

    fn on_create_node<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::CreateNode { event })?;

        Ok(())
    }

    fn on_pin_node(system: &mut SystemConfig<V>, node_id: &NodeId) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::PinNode { node_id })?;

        Ok(())
    }

    fn on_drop_node<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::DropNode { event })?;

        Ok(())
    }

    fn on_move_module<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &MoveModuleEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::MoveModule { event })?;

        Ok(())
    }

    fn on_open_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::OpenSubstate { event })?;

        Ok(())
    }

    fn on_mark_substate_as_transient(
        system: &mut SystemConfig<V>,
        node_id: &NodeId,
        partition_number: &PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(), RuntimeError> {
        system.modules.costing.apply_execution_cost(
            ExecutionCostingEntry::MarkSubstateAsTransient {
                node_id,
                partition_number,
                substate_key,
            },
        )?;

        Ok(())
    }

    fn on_read_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::ReadSubstate { event })?;

        Ok(())
    }

    fn on_write_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::WriteSubstate { event })?;

        Ok(())
    }

    fn on_close_substate<Y: KernelInternalApi<SystemConfig<V>>>(
        api: &mut Y,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::CloseSubstate { event })?;

        Ok(())
    }

    fn on_set_substate(
        system: &mut SystemConfig<V>,
        event: &SetSubstateEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::SetSubstate { event })?;

        Ok(())
    }

    fn on_remove_substate(
        system: &mut SystemConfig<V>,
        event: &RemoveSubstateEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::RemoveSubstate { event })?;

        Ok(())
    }

    fn on_scan_keys(
        system: &mut SystemConfig<V>,
        event: &ScanKeysEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::ScanKeys { event })?;

        Ok(())
    }

    fn on_drain_substates(
        system: &mut SystemConfig<V>,
        event: &DrainSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::DrainSubstates { event })?;

        Ok(())
    }

    fn on_scan_sorted_substates(
        system: &mut SystemConfig<V>,
        event: &ScanSortedSubstatesEvent,
    ) -> Result<(), RuntimeError> {
        system
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::ScanSortedSubstates { event })?;

        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(ExecutionCostingEntry::AllocateNodeId)?;

        Ok(())
    }
}
