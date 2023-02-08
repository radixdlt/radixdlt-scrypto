use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::kernel::{KernelModuleApi, LockFlags, ResolvedReceiver};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeModuleInit;
use crate::{
    errors::{CanBeAbortion, ModuleError, RuntimeError},
    kernel::KernelModule,
    kernel::{CallFrameUpdate, ResolvedActor},
    system::node::RENodeInit,
    transaction::AbortReason,
};
use radix_engine_interface::api::types::{
    FnIdentifier, GlobalAddress, GlobalOffset, LockHandle, NodeModuleId, RoyaltyOffset,
    SubstateOffset, VaultOffset,
};
use radix_engine_interface::{api::types::RENodeId, *};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
    let cost_units = base_price(&api.get_module_state().costing.fee_table);
    api.get_module_state()
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
    api.get_module_state()
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
        _fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.get_current_depth();

        if current_depth == api.get_module_state().costing.max_call_depth {
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

    fn before_new_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        callee: &ResolvedActor,
        _nodes_and_refs: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        match &callee.identifier {
            FnIdentifier::Native(native_fn) => apply_execution_cost(
                api,
                CostingReason::RunNative,
                |fee_table| fee_table.run_native_fn_cost(&native_fn),
                1,
            ),
            _ => Ok(()),
        }
    }

    fn after_actor_run<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        // Identify the function, and optional component address
        let actor = api.get_current_actor();
        let (scrypto_fn_identifier, optional_component_address) = match actor.identifier {
            FnIdentifier::Scrypto(scrypto_fn_identifier) => {
                let maybe_component = match &actor.receiver {
                    Some(ResolvedReceiver {
                        derefed_from:
                            Some((RENodeId::Global(GlobalAddress::Component(component_address)), ..)),
                        ..
                    }) => Some(*component_address),
                    _ => None,
                };

                (scrypto_fn_identifier, maybe_component)
            }
            _ => {
                return Ok(());
            }
        };

        //========================
        // Apply package royalty
        //========================

        let package_id = {
            let node_id = RENodeId::Global(GlobalAddress::Package(
                scrypto_fn_identifier.package_address,
            ));
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            let substate = api.get_ref(handle)?;
            let package_id = match substate.global_address() {
                GlobalAddressSubstate::Package(id) => *id,
                _ => panic!("Unexpected global address substate type"),
            };
            api.drop_lock(handle)?;

            package_id
        };

        let node_id = RENodeId::Package(package_id);
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::PackageRoyalty,
            offset,
            LockFlags::read_only(),
        )?;
        let substate = api.get_ref(handle)?;
        let royalty = substate
            .package_royalty_config()
            .royalty_config
            .get(&scrypto_fn_identifier.blueprint_name)
            .map(|x| x.get_rule(&scrypto_fn_identifier.ident).clone())
            .unwrap_or(0);
        api.drop_lock(handle)?;

        apply_royalty_cost(
            api,
            RoyaltyReceiver::Package(scrypto_fn_identifier.package_address, node_id),
            royalty,
        )?;

        // Pre-load accumulator and royalty vault substate to avoid additional substate loading
        // during track finalization.
        // TODO: refactor to defer substate loading to finalization.
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::PackageRoyalty,
            offset,
            LockFlags::MUTABLE,
        )?;
        let substate = api.get_ref(handle)?;
        {
            let royalty_vault = substate.package_royalty_accumulator().royalty.clone();
            let vault_node_id = RENodeId::Vault(royalty_vault.vault_id());
            let vault_handle = api.lock_substate(
                vault_node_id,
                NodeModuleId::SELF,
                SubstateOffset::Vault(VaultOffset::Vault),
                LockFlags::MUTABLE,
            )?;
            api.drop_lock(vault_handle)?;
        }
        api.drop_lock(handle)?;

        //========================
        // Apply component royalty
        //========================

        if let Some(component_address) = optional_component_address {
            let component_id = {
                let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
                let offset = SubstateOffset::Global(GlobalOffset::Global);
                let handle =
                    api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
                let substate = api.get_ref(handle)?;
                let component_id = match substate.global_address() {
                    GlobalAddressSubstate::Component(id) => *id,
                    _ => panic!("Unexpected global address substate type"),
                };
                api.drop_lock(handle)?;
                component_id
            };

            let node_id = RENodeId::Component(component_id);
            let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
            let handle = api.lock_substate(
                node_id,
                NodeModuleId::ComponentRoyalty,
                offset,
                LockFlags::read_only(),
            )?;
            let substate = api.get_ref(handle)?;
            let royalty = substate
                .component_royalty_config()
                .royalty_config
                .get_rule(&scrypto_fn_identifier.ident)
                .clone();
            apply_royalty_cost(
                api,
                RoyaltyReceiver::Component(component_address, node_id),
                royalty,
            )?;
            api.drop_lock(handle)?;

            // Pre-load accumulator and royalty vault substate to avoid additional substate loading
            // during track finalization.
            // TODO: refactor to defer substate loading to finalization.
            let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
            let handle = api.lock_substate(
                node_id,
                NodeModuleId::ComponentRoyalty,
                offset,
                LockFlags::MUTABLE,
            )?;
            let substate = api.get_ref(handle)?;
            {
                let royalty_vault = substate.component_royalty_accumulator().royalty.clone();
                let vault_node_id = RENodeId::Vault(royalty_vault.vault_id());
                let vault_handle = api.lock_substate(
                    vault_node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                    LockFlags::MUTABLE,
                )?;
                api.drop_lock(vault_handle)?;
            }
            api.drop_lock(handle)?;
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

    fn on_lock_substate<Y: KernelModuleApi<RuntimeError>>(
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

    fn on_wasm_instantiation<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        code: &[u8],
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::InstantiateWasm,
            |fee_table| fee_table.wasm_instantiation_per_byte(),
            code.len(),
        )
    }

    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        units: u32,
    ) -> Result<(), RuntimeError> {
        // We multiply by a large enough factor to ensure spin loops end within a fraction of a second.
        // These values will be tweaked, alongside the whole fee table.
        apply_execution_cost(api, CostingReason::RunWasm, |_| units, 5)
    }
}
