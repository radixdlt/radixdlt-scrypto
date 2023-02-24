use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelModuleApi, LockFlags};
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeModuleInit;
use crate::{
    errors::{CanBeAbortion, ModuleError, RuntimeError},
    system::node::RENodeInit,
    transaction::AbortReason,
};
use radix_engine_interface::api::types::{
    ComponentAddress, InvocationIdentifier, LockHandle, MethodIdentifier, NodeModuleId,
    RoyaltyOffset, SubstateOffset, VaultId, VaultOffset,
};
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::blueprints::resource::Resource;
use radix_engine_interface::constants::*;
use radix_engine_interface::{api::types::RENodeId, *};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
    MaxCallDepthLimitReached,
}

impl CanBeAbortion for CostingError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::FeeReserveError(err) => err.abortion(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CostingModule {
    pub fee_reserve: SystemLoanFeeReserve,
    pub fee_table: FeeTable,
    pub max_call_depth: usize,
}

impl CostingModule {
    pub fn take_fee_reserve(self) -> SystemLoanFeeReserve {
        self.fee_reserve
    }
}

fn apply_execution_cost<Y: KernelModuleApi<RuntimeError>, F>(
    api: &mut Y,
    reason: CostingReason,
    base_price: F,
    multiplier: usize,
) -> Result<(), RuntimeError>
where
    F: Fn(&FeeTable) -> u32,
{
    let cost_units = base_price(&api.kernel_get_module_state().costing.fee_table);
    api.kernel_get_module_state()
        .costing
        .fee_reserve
        .consume_multiplied_execution(cost_units, multiplier, reason)
        .map_err(|e| {
            RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(e)))
        })
}

fn apply_royalty_cost<Y: KernelModuleApi<RuntimeError>>(
    api: &mut Y,
    receiver: RoyaltyReceiver,
    amount: u32,
) -> Result<(), RuntimeError> {
    api.kernel_get_module_state()
        .costing
        .fee_reserve
        .consume_royalty(receiver, amount)
        .map_err(|e| {
            RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(e)))
        })
}

impl KernelModule for CostingModule {
    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _identifier: &InvocationIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.kernel_get_current_depth();

        if current_depth == api.kernel_get_module_state().costing.max_call_depth {
            return Err(RuntimeError::ModuleError(ModuleError::CostingError(
                CostingError::MaxCallDepthLimitReached,
            )));
        }

        if current_depth > 0 {
            apply_execution_cost(
                api,
                CostingReason::Invoke,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::Invoke {
                        input_size: input_size as u32,
                    })
                },
                1,
            )?;
        }

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        callee: &Option<ResolvedActor>,
        _nodes_and_refs: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        // Identify the function, and optional component address
        let (fn_identifier, optional_component) = match &callee {
            Some(ResolvedActor {
                     method: receiver,
                     fn_identifier: identifier,
            }) => {
                let maybe_component = match &receiver {
                    Some(MethodIdentifier(node_id, ..))
                        if matches!(
                            node_id,
                            RENodeId::Component(..)
                                | RENodeId::GlobalComponent(ComponentAddress::Normal(..))
                        ) =>
                    {
                        Some(node_id)
                    }
                    _ => None,
                };

                (identifier, maybe_component)
            }
            _ => {
                return Ok(());
            }
        };

        // FIXME: algin native packages with wasm package, or read package type info and disallow royalty on native package.
        let package_address = fn_identifier.package_address;
        if package_address == RESOURCE_MANAGER_PACKAGE
            || package_address == IDENTITY_PACKAGE
            || package_address == EPOCH_MANAGER_PACKAGE
            || package_address == CLOCK_PACKAGE
            || package_address == ACCOUNT_PACKAGE
            || package_address == ACCESS_CONTROLLER_PACKAGE
            || package_address == LOGGER_PACKAGE
            || package_address == TRANSACTION_RUNTIME_PACKAGE
            || package_address == AUTH_ZONE_PACKAGE
            || package_address == METADATA_PACKAGE
            || package_address == ROYALTY_PACKAGE
            || package_address == ACCESS_RULES_PACKAGE
            || package_address == PACKAGE_LOADER
        {
            return Ok(());
        }

        /*
         * Apply package royalty
         */
        let handle = api.kernel_lock_substate(
            RENodeId::GlobalPackage(package_address),
            NodeModuleId::PackageRoyalty,
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
            LockFlags::read_only(),
        )?;
        let substate = api.kernel_get_substate_ref(handle)?;
        let royalty_amount = substate
            .package_royalty_config()
            .royalty_config
            .get(&fn_identifier.blueprint_name)
            .map(|x| x.get_rule(&fn_identifier.ident).clone())
            .unwrap_or(0);
        api.kernel_drop_lock(handle)?;

        // TODO: refactor to defer substate loading to finalization.
        let handle = api.kernel_lock_substate(
            RENodeId::GlobalPackage(package_address),
            NodeModuleId::PackageRoyalty,
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
            LockFlags::MUTABLE,
        )?;
        let substate = api.kernel_get_substate_ref(handle)?;
        {
            let royalty_vault = substate.package_royalty_accumulator().royalty.clone();
            let vault_node_id = RENodeId::Vault(royalty_vault.vault_id());
            let vault_handle = api.kernel_lock_substate(
                vault_node_id,
                NodeModuleId::SELF,
                SubstateOffset::Vault(VaultOffset::Vault),
                LockFlags::MUTABLE,
            )?;
            api.kernel_drop_lock(vault_handle)?;
        }
        api.kernel_drop_lock(handle)?;

        apply_royalty_cost(
            api,
            RoyaltyReceiver::Package(fn_identifier.package_address),
            royalty_amount,
        )?;

        /*
         * Apply component royalty
         */
        if let Some(component_node_id) = optional_component {
            let handle = api.kernel_lock_substate(
                *component_node_id,
                NodeModuleId::ComponentRoyalty,
                SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                LockFlags::read_only(),
            )?;
            let substate = api.kernel_get_substate_ref(handle)?;
            let royalty_amount = substate
                .component_royalty_config()
                .royalty_config
                .get_rule(&fn_identifier.ident)
                .clone();
            api.kernel_drop_lock(handle)?;

            // TODO: refactor to defer substate loading to finalization.
            let handle = api.kernel_lock_substate(
                *component_node_id,
                NodeModuleId::ComponentRoyalty,
                SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                LockFlags::MUTABLE,
            )?;
            let substate = api.kernel_get_substate_ref(handle)?;
            {
                let royalty_vault = substate.component_royalty_accumulator().royalty.clone();
                let vault_node_id = RENodeId::Vault(royalty_vault.vault_id());
                let vault_handle = api.kernel_lock_substate(
                    vault_node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                    LockFlags::MUTABLE,
                )?;
                api.kernel_drop_lock(vault_handle)?;
            }
            api.kernel_drop_lock(handle)?;

            apply_royalty_cost(
                api,
                RoyaltyReceiver::Component(*component_node_id),
                royalty_amount,
            )?;
        }

        Ok(())
    }

    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            api,
            CostingReason::CreateNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::CreateNode { size: 0 }),
            1,
        )?;
        Ok(())
    }

    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            api,
            CostingReason::DropNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode { size: 0 }),
            1,
        )?;

        Ok(())
    }

    fn before_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::LockSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::LockSubstate),
            1,
        )?;
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::ReadSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::ReadSubstate { size: size as u32 }),
            1,
        )?;
        Ok(())
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::WriteSubstate,
            |fee_table| {
                fee_table.kernel_api_cost(CostingEntry::WriteSubstate { size: size as u32 })
            },
            1,
        )?;
        Ok(())
    }

    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::DropLock,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
            1,
        )?;
        Ok(())
    }

    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        // We multiply by a large enough factor to ensure spin loops end within a fraction of a second.
        // These values will be tweaked, alongside the whole fee table.
        apply_execution_cost(
            api,
            match reason {
                ClientCostingReason::RunWasm => CostingReason::RunWasm,
                ClientCostingReason::RunPrecompiled => CostingReason::RunPrecompiled,
            },
            |_| units,
            5,
        )
    }

    fn on_credit_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        let changes = api
            .kernel_get_module_state()
            .costing
            .fee_reserve
            .lock_fee(vault_id, fee, contingent)
            .map_err(|e| {
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })?;
        Ok(changes)
    }
}
