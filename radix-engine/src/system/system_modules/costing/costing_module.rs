use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::blueprints::package::PackageRoyaltyNativeBlueprint;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::node_modules::royalty::ComponentRoyaltyBlueprint;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::track::interface::{StoreAccess, StoreAccessInfo};
use crate::types::*;
use crate::{
    errors::{CanBeAbortion, RuntimeError, SystemModuleError},
    transaction::AbortReason,
};
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::blueprints::package::BlueprintVersionKey;
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
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
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
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
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
        let fee_reserve = &mut costing.fee_reserve;
        let fee_table = &costing.fee_table;

        fee_reserve
            .consume_deferred(fee_table.tx_base_cost(), 1, CostingReason::TxBaseCost)
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_payload_cost_per_byte(),
                    costing.payload_len,
                    CostingReason::TxPayloadCost,
                )
            })
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_signature_verification_cost_per_sig(),
                    costing.num_of_signatures,
                    CostingReason::TxSignatureVerification,
                )
            })
            .map_err(|e| {
                RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                    CostingError::FeeReserveError(e),
                ))
            })
    }

    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.kernel_get_current_depth();
        if current_depth == api.kernel_get_system().modules.costing.max_call_depth {
            return Err(RuntimeError::SystemModuleError(
                SystemModuleError::CostingError(CostingError::MaxCallDepthLimitReached),
            ));
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

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICIENT_CPU: u64 = 335;
const COSTING_COEFFICIENT_CPU_DIV_BITS: u64 = 4; // used to divide by shift left operator
const COSTING_COEFFICIENT_CPU_DIV_BITS_ADDON: u64 = 6; // used to scale up or down all cpu instruction costing

const COSTING_COEFFICIENT_STORAGE: u64 = 10;
const COSTING_COEFFICIENT_STORAGE_DIV_BITS: u64 = 6; // used to scale up or down all storage costing

pub enum CostingEntry<'a> {
    /* invoke */
    Invoke {
        input_size: u32,
        actor: &'a Actor,
    },

    /* node */
    CreateNode {
        node_id: &'a NodeId,
    },
    DropNode,
    AllocateNodeId,

    /* substate */
    LockSubstate {
        node_id: &'a NodeId,
        partition_num: &'a PartitionNumber,
        substate_key: &'a SubstateKey,
    },
    ReadSubstate {
        size: u32,
    },
    WriteSubstate {
        size: u32,
    },
    ScanSubstate,
    SetSubstate,
    TakeSubstate,
    DropLock,
    SubstateReadFromDb {
        size: u32,
    },
    SubstateReadFromDbNotFound,
    SubstateReadFromTrack {
        size: u32,
    },
    SubstateWriteToTrack {
        size: u32,
    },
    SubstateRewriteToTrack {
        size_old: u32,
        size_new: u32,
    },
    // FIXME: more costing after API becomes stable.
}

/// CPU instructions usage numbers obtained from test runs with 'resource_tracker` feature enabled
/// and transformed (classified and groupped) using convert.py script.
fn kernel_api_cost_cpu_usage(&self, entry: &CostingEntry) -> u32 {
    ((match entry {
        CostingEntry::AllocateNodeId => 212,
        CostingEntry::CreateNode { node_id } => match node_id.entity_type() {
            Some(EntityType::GlobalAccessController) => 1736,
            Some(EntityType::GlobalAccount) => 1640,
            Some(EntityType::GlobalConsensusManager) => 1203,
            Some(EntityType::GlobalFungibleResourceManager) => 1160,
            Some(EntityType::GlobalGenericComponent) => 2370,
            Some(EntityType::GlobalIdentity) => 838,
            Some(EntityType::GlobalNonFungibleResourceManager) => 1587,
            Some(EntityType::GlobalPackage) => 1493,
            Some(EntityType::GlobalValidator) => 2374,
            Some(EntityType::GlobalVirtualSecp256k1Account) => 1590,
            Some(EntityType::GlobalVirtualSecp256k1Identity) => 906,
            Some(EntityType::InternalAccount) => 329,
            Some(EntityType::InternalFungibleVault) => 368,
            Some(EntityType::InternalGenericComponent) => 336,
            Some(EntityType::InternalKeyValueStore) => 828,
            Some(EntityType::InternalNonFungibleVault) => 356,
            _ => 1182, // average of above values
        },
        CostingEntry::DropLock => 114,
        CostingEntry::DropNode => 324, // average of gathered data
        CostingEntry::Invoke {
            input_size,
            actor: identifier,
        } => {
            let FnIdentifier {
                blueprint_id: blueprint,
                ident,
            } = identifier.fn_identifier();
            match &ident {
                FnIdent::Application(fn_name) => {
                    match (blueprint.blueprint_name.as_str(), fn_name.as_str()) {
                        ("Package", "publish_native") => (input_size * 13 + 10910) >> 2, // calculated using linear regression on gathered data
                        ("Package", "publish_wasm_advanced") => input_size * 22 + 289492, // calculated using linear regression on gathered data
                        _ => 411524, // average of above values without Package::publish_native and Package::publish_wasm_advanced
                    }
                }
                FnIdent::System(value) => {
                    match (blueprint.blueprint_name.as_str(), value) {
                        ("Identity", 0) => 252633,
                        ("Account", 0) => 220211,
                        _ => 236422, // average of above values
                    }
                }
            }
        }
        // FIXME update numbers below
        CostingEntry::LockSubstate {
            node_id: _,
            partition_num: _,
            substate_key: _,
        } => 100,
        CostingEntry::ScanSubstate => 16,
        CostingEntry::SetSubstate => 16,
        CostingEntry::TakeSubstate => 16,
        CostingEntry::ReadSubstate { size: _ } => 174,
        CostingEntry::WriteSubstate { size: _ } => 126,

        // following variants are used in storage usage part only
        CostingEntry::SubstateReadFromDb { size: _ } => 0,
        CostingEntry::SubstateReadFromDbNotFound => 0,
        CostingEntry::SubstateReadFromTrack { size: _ } => 0,
        CostingEntry::SubstateWriteToTrack { size: _ } => 0,
        CostingEntry::SubstateRewriteToTrack {
            size_old: _,
            size_new: _,
        } => 0,
    }) as u64
        * COSTING_COEFFICIENT_CPU
        >> (COSTING_COEFFICIENT_CPU_DIV_BITS + COSTING_COEFFICIENT_CPU_DIV_BITS_ADDON)) as u32
}

fn kernel_api_cost_storage_usage(&self, entry: &CostingEntry) -> u32 {
    ((match entry {
        CostingEntry::Invoke {
            input_size,
            actor: _,
        } => 10 * input_size,
        CostingEntry::SubstateReadFromDb { size } => {
            if *size <= 25 * 1024 {
                // apply constant value
                400u32
            } else {
                // apply function: f(size) = 0.0009622109 * size + 389.5155
                // approximated integer representation: f(size) = (63 * size) / 2^16 + 390
                let mut value: u64 = *size as u64;
                value *= 63; // 0.0009622109 << 16
                value += (value >> 16) + 390;
                value.try_into().unwrap_or(u32::MAX)
            }
        }
        CostingEntry::SubstateReadFromDbNotFound => 322, // average value from benchmark
        // FIXME: update numbers below
        CostingEntry::SubstateReadFromTrack { size } => 10 * size, // todo: determine correct value
        CostingEntry::SubstateWriteToTrack { size } => 10 * size,  // todo: determine correct value
        CostingEntry::SubstateRewriteToTrack {
            size_old: _,
            size_new,
        } => 10 * size_new, // todo: determine correct value
        _ => 0,
    }) as u64
        * COSTING_COEFFICIENT_STORAGE
        >> COSTING_COEFFICIENT_STORAGE_DIV_BITS) as u32
}

pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
    self.kernel_api_cost_cpu_usage(&entry) + self.kernel_api_cost_storage_usage(&entry)
}
