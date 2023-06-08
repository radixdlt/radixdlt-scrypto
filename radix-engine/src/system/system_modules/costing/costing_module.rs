use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::system::SystemService;
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::track::interface::{StoreAccess, StoreAccessInfo};
use crate::types::*;
use crate::{
    errors::{CanBeAbortion, ModuleError, RuntimeError},
    transaction::AbortReason,
};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::blueprints::package::PackageRoyaltySubstate;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::{types::NodeId, *};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
    MaxCallDepthLimitReached,
    WrongSubstateStoreDbAccessInfo,
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

    fn apply_access_store_costs(
        &mut self,
        costing_reason: CostingReason,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        for item in store_access.data().iter() {
            match item {
                StoreAccess::ReadFromDb(size) => self.apply_execution_cost(
                    costing_reason.clone(),
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::SubstateReadFromDb {
                            size: *size as u32,
                        })
                    },
                    1,
                )?,
                StoreAccess::ReadFromTrack(size) => self.apply_execution_cost(
                    costing_reason.clone(),
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::SubstateReadFromTrack {
                            size: *size as u32,
                        })
                    },
                    1,
                )?,
                StoreAccess::WriteToTrack(size) => self.apply_execution_cost(
                    costing_reason.clone(),
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::SubstateWriteToTrack {
                            size: *size as u32,
                        })
                    },
                    1,
                )?,
                StoreAccess::RewriteToTrack(size_old, size_new) => self.apply_execution_cost(
                    costing_reason.clone(),
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::SubstateRewriteToTrack {
                            size_old: *size_old as u32,
                            size_new: *size_new as u32,
                        })
                    },
                    1,
                )?,
                StoreAccess::ReadFromDbNotFound => self.apply_execution_cost(
                    costing_reason.clone(),
                    |fee_table| fee_table.kernel_api_cost(CostingEntry::SubstateReadFromDbNotFound),
                    1,
                )?,
            }
        }
        Ok(())
    }
}

fn apply_royalty_cost<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
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
            RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(e)))
        })
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for CostingModule {
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let costing = &mut api.kernel_get_system().modules.costing;
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

    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.kernel_get_current_depth();
        if current_depth == api.kernel_get_system().modules.costing.max_call_depth {
            return Err(RuntimeError::ModuleError(ModuleError::CostingError(
                CostingError::MaxCallDepthLimitReached,
            )));
        }

        if current_depth > 0 {
            api.kernel_get_system()
                .modules
                .costing
                .apply_execution_cost(
                    CostingReason::Invoke,
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::Invoke {
                            input_size: invocation.len() as u32,
                            actor: &invocation.actor,
                        })
                    },
                    1,
                )?;
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
            let blueprint = callee.blueprint();
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
        let handle = api.kernel_lock_substate(
            blueprint.package_address.as_node_id(),
            OBJECT_BASE_PARTITION,
            &PackageField::Royalty.into(),
            LockFlags::MUTABLE,
            SystemLockData::default(),
        )?;
        let mut substate: PackageRoyaltySubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        let royalty_charge = substate
            .blueprint_royalty_configs
            .get(blueprint.blueprint_name.as_str())
            .map(|x| x.get_rule(ident).clone())
            .unwrap_or(RoyaltyAmount::Free);
        if royalty_charge.is_non_zero() {
            let vault_id = if let Some(vault) = substate.royalty_vault {
                vault
            } else {
                let mut system = SystemService::new(api);
                let new_vault = ResourceManager(RADIX_TOKEN).new_empty_vault(&mut system)?;
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
                ROYALTY_FIELD_PARTITION,
                &RoyaltyField::RoyaltyConfig.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let substate: ComponentRoyaltyConfigSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let royalty_charge = substate.royalty_config.get_rule(ident).clone();
            api.kernel_drop_lock(handle)?;

            if royalty_charge.is_non_zero() {
                let handle = api.kernel_lock_substate(
                    component_address.as_node_id(),
                    ROYALTY_FIELD_PARTITION,
                    &RoyaltyField::RoyaltyAccumulator.into(),
                    LockFlags::MUTABLE,
                    SystemLockData::default(),
                )?;
                let mut substate: ComponentRoyaltyAccumulatorSubstate =
                    api.kernel_read_substate(handle)?.as_typed().unwrap();
                let vault_id = if let Some(vault) = substate.royalty_vault {
                    vault
                } else {
                    let mut system = SystemService::new(api);
                    let new_vault = ResourceManager(RADIX_TOKEN).new_empty_vault(&mut system)?;
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

    fn after_create_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::CreateNode,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::CreateNode { node_id }),
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::CreateNode, store_access)
    }

    fn after_drop_node<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::DropNode,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode),
                1,
            )
    }

    fn before_lock_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::LockSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::LockSubstate {
                        node_id,
                        partition_num,
                        substate_key,
                    })
                },
                1,
            )
    }

    fn after_lock_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _handle: LockHandle,
        store_access: &StoreAccessInfo,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::LockSubstate, store_access)
    }

    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part + value size costing
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::ReadSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::ReadSubstate {
                        size: value_size as u32,
                    })
                },
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::ReadSubstate, store_access)
    }

    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part + value size costing
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::WriteSubstate,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::WriteSubstate {
                        size: value_size as u32,
                    })
                },
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::WriteSubstate, store_access)
    }

    fn on_drop_lock<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::DropLock,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::DropLock, store_access)
    }

    fn on_scan_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::ScanSubstate,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::ScanSubstate),
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::ScanSubstate, store_access)
    }

    fn on_set_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::SetSubstate,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::SetSubstate),
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::SetSubstate, store_access)
    }

    fn on_take_substates<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        // CPU execution part
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::TakeSubstate,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::TakeSubstate),
                1,
            )?;
        // Storage usage part
        api.kernel_get_system()
            .modules
            .costing
            .apply_access_store_costs(CostingReason::TakeSubstate, store_access)
    }

    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
                CostingReason::AllocateNodeId,
                |fee_table| fee_table.kernel_api_cost(CostingEntry::AllocateNodeId),
                1,
            )
    }
}
