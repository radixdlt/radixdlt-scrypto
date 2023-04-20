use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::system::SystemDownstream;
use crate::system::system_callback::{SystemCallback, SystemInvocation};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::{
    errors::{CanBeAbortion, ModuleError, RuntimeError},
    transaction::AbortReason,
};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate,
};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::package::PackageRoyaltySubstate;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::{types::NodeId, *};
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
    pub payload_len: usize,
    pub num_of_signatures: usize,
}

impl CostingModule {
    pub fn fee_reserve(self) -> SystemLoanFeeReserve {
        self.fee_reserve
    }

    pub fn apply_execution_cost<F>(
        &mut self,
        reason: CostingReason,
        base_price: F,
        multiplier: usize,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&FeeTable) -> u32,
    {
        let cost_units = base_price(&self.fee_table);
        self.fee_reserve
            .consume_multiplied_execution(cost_units, multiplier, reason)
            .map_err(|e| {
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })
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
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })
    }
}

fn apply_royalty_cost<Y: KernelApi<SystemCallback<V>>, V: SystemCallbackObject>(
    api: &mut Y,
    cost_units: u32,
    recipient: RoyaltyRecipient,
    recipient_vault_id: NodeId,
) -> Result<(), RuntimeError> {
    api.kernel_get_callback()
        .modules
        .costing
        .fee_reserve
        .consume_royalty(cost_units, recipient, recipient_vault_id)
        .map_err(|e| {
            RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(e)))
        })
}

impl<V: SystemCallbackObject> SystemModule<SystemCallback<V>> for CostingModule {
    fn on_init<Y: KernelApi<SystemCallback<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let costing = &mut api.kernel_get_callback().modules.costing;
        let fee_reserve = &mut costing.fee_reserve;
        let fee_table = &costing.fee_table;

        fee_reserve
            .consume_deferred(fee_table.tx_base_fee(), 1, CostingReason::TxBaseCost)
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_payload_cost_per_byte(),
                    costing.payload_len,
                    CostingReason::TxPayloadCost,
                )
            })
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_signature_verification_per_sig(),
                    costing.num_of_signatures,
                    CostingReason::TxSignatureVerification,
                )
            })
            .map_err(|e| {
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })
    }

    fn before_invoke<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        identifier: &KernelInvocation<SystemInvocation>,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.kernel_get_current_depth();
        if current_depth == api.kernel_get_callback().modules.costing.max_call_depth {
            return Err(RuntimeError::ModuleError(ModuleError::CostingError(
                CostingError::MaxCallDepthLimitReached,
            )));
        }

        if current_depth > 0 {
            api.kernel_get_callback()
                .modules
                .costing
                .apply_execution_cost(
                    CostingReason::Invoke,
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::Invoke {
                            input_size: input_size as u32,
                            identifier: &identifier.sys_invocation,
                        })
                    },
                    1,
                )?;
        }

        Ok(())
    }

    fn before_push_frame<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        callee: &Actor,
        _nodes_and_refs: &mut CallFrameUpdate,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        // Identify the function, and optional component address
        let (blueprint, ident, optional_component) = {
            let blueprint = callee.blueprint();
            let (maybe_component, ident) = match &callee {
                Actor::Method { node_id, ident, .. } => {
                    if node_id.is_global_component() {
                        (
                            Some(ComponentAddress::new_unchecked(node_id.clone().into())),
                            ident,
                        )
                    } else {
                        (None, ident)
                    }
                }
                Actor::Function { ident, .. } => (None, ident),
                Actor::VirtualLazyLoad { .. } => {
                    return Ok(());
                }
            };

            (blueprint, ident, maybe_component)
        };

        //===========================
        // Apply package royalty
        //===========================
        let handle = api.kernel_lock_substate(
            blueprint.package_address.as_node_id(),
            SysModuleId::Object.into(),
            &PackageOffset::Royalty.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: PackageRoyaltySubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        let royalty_charge = substate
            .blueprint_royalty_configs
            .get(blueprint.blueprint_name.as_str())
            .map(|x| x.get_rule(ident).clone())
            .unwrap_or(0);
        if royalty_charge > 0 {
            let vault_id = if let Some(vault) = substate.royalty_vault {
                vault
            } else {
                let mut system = SystemDownstream::new(api);
                let new_vault = ResourceManager(RADIX_TOKEN).new_vault(&mut system)?;
                substate.royalty_vault = Some(new_vault);
                api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&substate))?;
                new_vault
            };
            apply_royalty_cost(
                api,
                royalty_charge,
                RoyaltyRecipient::Package(blueprint.package_address),
                vault_id.0,
            )?;
        }
        api.kernel_drop_lock(handle)?;

        //===========================
        // Apply component royalty
        //===========================
        if let Some(component_address) = optional_component {
            let handle = api.kernel_lock_substate(
                component_address.as_node_id(),
                SysModuleId::Royalty.into(),
                &RoyaltyOffset::RoyaltyConfig.into(),
                LockFlags::read_only(),
            )?;
            let substate: ComponentRoyaltyConfigSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let royalty_charge = substate.royalty_config.get_rule(ident).clone();
            api.kernel_drop_lock(handle)?;

            if royalty_charge > 0 {
                let handle = api.kernel_lock_substate(
                    component_address.as_node_id(),
                    SysModuleId::Royalty.into(),
                    &RoyaltyOffset::RoyaltyAccumulator.into(),
                    LockFlags::MUTABLE,
                )?;
                let mut substate: ComponentRoyaltyAccumulatorSubstate =
                    api.kernel_read_substate(handle)?.as_typed().unwrap();
                let vault_id = if let Some(vault) = substate.royalty_vault {
                    vault
                } else {
                    let mut system = SystemDownstream::new(api);
                    let new_vault = ResourceManager(RADIX_TOKEN).new_vault(&mut system)?;
                    substate.royalty_vault = Some(new_vault);
                    new_vault
                };
                apply_royalty_cost(
                    api,
                    royalty_charge,
                    RoyaltyRecipient::Component(component_address.clone()),
                    vault_id.into(),
                )?;
                api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&substate))?;
                api.kernel_drop_lock(handle)?;
            }
        }

        Ok(())
    }

    fn before_create_node<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        _node_module_init: &BTreeMap<ModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::CreateNode,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::CreateNode { size: 0, node_id })
                },
                1,
            )?;
        Ok(())
    }

    fn after_drop_node<Y: KernelApi<SystemCallback<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        // TODO: calculate size
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::DropNode,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode { size: 0 }),
                1,
            )?;

        Ok(())
    }

    fn before_lock_substate<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        module_id: &ModuleId,
        substate_key: &SubstateKey,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::LockSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::LockSubstate {
                        node_id,
                        module_id: &SysModuleId::from_repr(module_id.0).unwrap(),
                        substate_key,
                    })
                },
                1,
            )?;
        Ok(())
    }

    fn on_read_substate<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::ReadSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::ReadSubstate { size: size as u32 })
                },
                1,
            )?;
        Ok(())
    }

    fn on_write_substate<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::WriteSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::WriteSubstate { size: size as u32 })
                },
                1,
            )?;
        Ok(())
    }

    fn on_drop_lock<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::DropLock,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
                1,
            )?;
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<SystemCallback<V>>>(
        api: &mut Y,
        _entity_type: Option<EntityType>,
        virtual_node: bool,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_callback()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::AllocateNodeId,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::AllocateNodeId { virtual_node })
                },
                1,
            )?;
        Ok(())
    }
}
