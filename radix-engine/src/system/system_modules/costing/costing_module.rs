use super::*;
use super::{FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::blueprints::package::PackageRoyaltyNativeBlueprint;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::node_modules::royalty::ComponentRoyaltyBlueprint;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::track::interface::{StoreAccessInfo, StoreCommit};
use crate::types::*;
use crate::{
    errors::{CanBeAbortion, RuntimeError, SystemModuleError},
    transaction::AbortReason,
};
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
    pub payload_len: usize,
    pub num_of_signatures: usize,
    /// The maximum allowed method royalty in XRD allowed to be set by package and component owners
    pub max_per_function_royalty_in_xrd: Decimal,
    pub enable_cost_breakdown: bool,
    pub costing_traces: IndexMap<String, u32>,
}

impl CostingModule {
    pub fn fee_reserve(self) -> SystemLoanFeeReserve {
        self.fee_reserve
    }

    pub fn apply_execution_cost(
        &mut self,
        costing_entry: CostingEntry,
    ) -> Result<(), RuntimeError> {
        let cost_units = costing_entry.to_cost_units(&self.fee_table);

        self.fee_reserve
            .consume_execution(cost_units)
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })?;

        if self.enable_cost_breakdown {
            let key = costing_entry.to_trace_key();
            self.costing_traces
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }

        Ok(())
    }

    pub fn apply_deferred_execution_cost(
        &mut self,
        costing_entry: CostingEntry,
    ) -> Result<(), RuntimeError> {
        let cost_units = costing_entry.to_cost_units(&self.fee_table);

        self.fee_reserve.consume_deferred(cost_units).map_err(|e| {
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(e),
            ))
        })?;

        if self.enable_cost_breakdown {
            let key = costing_entry.to_trace_key();
            self.costing_traces
                .entry(key)
                .or_default()
                .add_assign(cost_units);
        }

        Ok(())
    }

    pub fn apply_state_expansion_cost(
        &mut self,
        store_commit: &StoreCommit,
    ) -> Result<(), RuntimeError> {
        self.fee_reserve
            .consume_state_expansion(store_commit)
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
    recipient_vault_id: NodeId,
) -> Result<(), RuntimeError> {
    api.kernel_get_system()
        .modules
        .costing
        .fee_reserve
        .consume_royalty(royalty_amount, recipient, recipient_vault_id)
        .map_err(|e| {
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(e),
            ))
        })
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for CostingModule {
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let costing = &mut api.kernel_get_system().modules.costing;

        costing.apply_deferred_execution_cost(CostingEntry::TxBaseCost)?;
        costing.apply_deferred_execution_cost(CostingEntry::TxPayloadCost {
            size: costing.payload_len,
        })?;
        costing.apply_deferred_execution_cost(CostingEntry::TxSignatureVerification {
            num_signatures: costing.num_of_signatures,
        })?;

        Ok(())
    }

    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        // Skip invocation costing for transaction processor
        if api.kernel_get_current_depth() > 0 {
            api.kernel_get_system()
                .modules
                .costing
                .apply_execution_cost(CostingEntry::BeforeInvoke {
                    actor: &invocation.actor,
                    input_size: invocation.len(),
                })?;
        }

        Ok(())
    }

    fn after_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        // Skip invocation costing for transaction processor
        if api.kernel_get_current_depth() > 0 {
            api.kernel_get_system()
                .modules
                .costing
                .apply_execution_cost(CostingEntry::AfterInvoke { output_size })?;
        }

        Ok(())
    }

    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        _message: &mut Message,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        // Identify the function, and optional component address
        let (blueprint, ident, optional_component) = {
            let blueprint = callee.blueprint_id();
            let (maybe_component, ident) = match &callee {
                Actor::Method(MethodActor { node_id, ident, .. }) => {
                    if node_id.is_global_component() {
                        (
                            Some(ComponentAddress::new_or_panic(node_id.clone().into())),
                            ident,
                        )
                    } else {
                        (None, ident)
                    }
                }
                Actor::Function { ident, .. } => (None, ident),
                Actor::VirtualLazyLoad { .. } | Actor::Root => {
                    return Ok(());
                }
            };

            (blueprint, ident, maybe_component)
        };

        //===========================
        // Apply package royalty
        //===========================
        let bp_version_key = BlueprintVersionKey::new_default(blueprint.blueprint_name.as_str());
        PackageRoyaltyNativeBlueprint::charge_package_royalty(
            blueprint.package_address.as_node_id(),
            &bp_version_key,
            ident,
            api,
        )?;

        //===========================
        // Apply component royalty
        //===========================
        if let Some(component_address) = optional_component {
            ComponentRoyaltyBlueprint::charge_component_royalty(
                component_address.as_node_id(),
                ident,
                api,
            )?;
        }

        Ok(())
    }

    fn after_create_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::CreateNode {
                node_id,
                total_substate_size,
                store_access: store_access,
            })?;

        Ok(())
    }

    #[inline(always)]
    fn after_move_modules<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _src_node_id: &NodeId,
        _dest_node_id: &NodeId,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::MoveModules { store_access })?;

        Ok(())
    }

    fn after_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        total_substate_size: usize,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::DropNode {
                total_substate_size,
            })?;

        Ok(())
    }

    fn after_open_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _handle: LockHandle,
        node_id: &NodeId,
        store_access: &StoreAccessInfo,
        value_size: usize,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::OpenSubstate {
                node_id,
                store_access,
                value_size,
            })?;

        Ok(())
    }

    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::ReadSubstate {
                value_size,
                store_access: store_access,
            })?;

        Ok(())
    }

    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::WriteSubstate {
                value_size,
                store_access: &store_access,
            })?;

        Ok(())
    }

    fn on_close_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::CloseSubstate {
                store_access: store_access,
            })?;

        Ok(())
    }

    fn on_scan_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::ScanSubstates {
                store_access: store_access,
            })?;

        Ok(())
    }

    fn on_set_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::SetSubstate {
                value_size,
                store_access: store_access,
            })?;

        Ok(())
    }

    fn on_take_substates<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::TakeSubstate {
                store_access: store_access,
            })?;

        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(CostingEntry::AllocateNodeId)?;

        Ok(())
    }
}
