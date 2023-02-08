use super::{FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::{
    errors::{CanBeAbortion, RuntimeError},
    kernel::{kernel_api::KernelSubstateApi, KernelModule, KernelNodeApi},
    kernel::{CallFrameUpdate, ResolvedActor},
    system::node::RENodeInit,
    transaction::AbortReason,
};
use radix_engine_interface::{
    api::types::{RENodeId, RENodeType},
    *,
};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
}

impl CanBeAbortion for CostingError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::FeeReserveError(err) => err.abortion(),
            _ => None,
        }
    }
}

pub struct CostingModule;

fn apply_execution_cost<F>(
    heap: &mut Heap,
    reason: CostingReason,
    base_price: F,
    multiplier: usize,
) -> Result<(), RuntimeError>
where
    F: Fn(&FeeTable) -> u32,
{
    if let Ok(mut substate) = heap.get_substate_mut(
        RENodeId::FeeReserve,
        NodeModuleId::SELF,
        &SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
    ) {
        let fee_reserve_substate = substate.fee_reserve();

        let cost_units = base_price(&fee_reserve_substate.fee_table);
        fee_reserve_substate
            .fee_reserve
            .consume_multiplied_execution(cost_units, multiplier, reason)
            .map_err(|e| {
                RuntimeError::ExecutionCostingError(ExecutionCostingError::CostingError(e))
            })
    } else {
        Ok(())
    }
}

fn apply_royalty_cost(
    heap: &mut Heap,
    receiver: RoyaltyReceiver,
    amount: u32,
) -> Result<(), RuntimeError> {
    if let Ok(mut substate) = heap.get_substate_mut(
        RENodeId::FeeReserve,
        NodeModuleId::SELF,
        &SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
    ) {
        let fee_reserve_substate = substate.fee_reserve();

        fee_reserve_substate
            .fee_reserve
            .consume_royalty(receiver, amount)
            .map_err(|e| RuntimeError::RoyaltyCostingError(RoyaltyCostingError::CostingError(e)))
    } else {
        Ok(())
    }

    // Identify the function, and optional component address
    let (scrypto_fn_identifier, optional_component_address) = match &callee.identifier {
        FnIdentifier::Scrypto(scrypto_fn_identifier) => {
            let maybe_component = match &callee.receiver {
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
        track
            .acquire_lock(
                SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                LockFlags::read_only(),
            )
            .map_err(RoyaltyCostingError::from)?;
        let substate = track.get_substate(node_id, NodeModuleId::SELF, &offset);
        let package_id = match substate.global_address() {
            GlobalAddressSubstate::Package(id) => *id,
            _ => panic!("Unexpected global address substate type"),
        };
        track
            .release_lock(
                SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                false,
            )
            .map_err(RoyaltyCostingError::from)?;
        package_id
    };

    let node_id = RENodeId::Package(package_id);
    let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
    track
        .acquire_lock(
            SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
            LockFlags::read_only(),
        )
        .map_err(RoyaltyCostingError::from)?;
    let substate = track.get_substate(node_id, NodeModuleId::PackageRoyalty, &offset);
    let royalty = substate
        .package_royalty_config()
        .royalty_config
        .get(&scrypto_fn_identifier.blueprint_name)
        .map(|x| x.get_rule(&scrypto_fn_identifier.ident).clone())
        .unwrap_or(0);
    apply_royalty_cost(
        heap,
        RoyaltyReceiver::Package(scrypto_fn_identifier.package_address, node_id),
        royalty,
    )?;
    track
        .release_lock(
            SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
            false,
        )
        .map_err(RoyaltyCostingError::from)?;

    // Pre-load accumulator and royalty vault substate to avoid additional substate loading
    // during track finalization.
    // TODO: refactor to defer substate loading to finalization.
    let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
    track
        .acquire_lock(
            SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
            LockFlags::MUTABLE,
        )
        .map_err(RoyaltyCostingError::from)?;
    let royalty_vault = track
        .get_substate(node_id, NodeModuleId::PackageRoyalty, &offset)
        .package_royalty_accumulator()
        .royalty
        .clone();
    preload_vault!(track, royalty_vault);
    track
        .release_lock(
            SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
            false,
        )
        .map_err(RoyaltyCostingError::from)?;

    //========================
    // Apply component royalty
    //========================

    if let Some(component_address) = optional_component_address {
        let component_id = {
            let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            track
                .acquire_lock(
                    SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                    LockFlags::read_only(),
                )
                .map_err(RoyaltyCostingError::from)?;
            let substate = track.get_substate(node_id, NodeModuleId::SELF, &offset);
            let component_id = match substate.global_address() {
                GlobalAddressSubstate::Component(id) => *id,
                _ => panic!("Unexpected global address substate type"),
            };
            track
                .release_lock(
                    SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                    false,
                )
                .map_err(RoyaltyCostingError::from)?;
            component_id
        };

        let node_id = RENodeId::Component(component_id);
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
        track
            .acquire_lock(
                SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                LockFlags::read_only(),
            )
            .map_err(RoyaltyCostingError::from)?;
        let substate = track.get_substate(node_id, NodeModuleId::ComponentRoyalty, &offset);
        let royalty = substate
            .component_royalty_config()
            .royalty_config
            .get_rule(&scrypto_fn_identifier.ident)
            .clone();
        apply_royalty_cost(
            heap,
            RoyaltyReceiver::Component(component_address, node_id),
            royalty,
        )?;
        track
            .release_lock(
                SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                false,
            )
            .map_err(RoyaltyCostingError::from)?;

        // Pre-load accumulator and royalty vault substate to avoid additional substate loading
        // during track finalization.
        // TODO: refactor to defer substate loading to finalization.
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
        track
            .acquire_lock(
                SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                LockFlags::MUTABLE,
            )
            .map_err(RoyaltyCostingError::from)?;
        let royalty_vault = track
            .get_substate(node_id, NodeModuleId::ComponentRoyalty, &offset)
            .component_royalty_accumulator()
            .royalty
            .clone();
        preload_vault!(track, royalty_vault);
        track
            .release_lock(
                SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                false,
            )
            .map_err(RoyaltyCostingError::from)?;
    }
}

impl KernelModule for CostingModule {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> Result<(), RuntimeError> {
        let node_id = api.allocate_node_id(RENodeType::FeeReserve)?;
        api.create_node(
            node_id,
            RENodeInit::FeeReserve(FeeReserveSubstate {
                fee_reserve,
                fee_table,
            }),
            BTreeMap::new(),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<FeeReserveSubstate, RuntimeError> {
        let substate: FeeReserveSubstate = api.drop_node(RENodeId::FeeReserve)?.into();

        Ok(substate)
    }

    pub fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if api.get_visible_node_data(RENodeId::FeeReserve).is_ok() {
            call_frame_update
                .node_refs_to_copy
                .insert(RENodeId::FeeReserve);
        }

        Ok(())
    }

    fn pre_kernel_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        if current_frame.depth == self.max_depth {
            return Err(RuntimeError::ExecutionCostingError(
                ExecutionCostingError::MaxCallDepthLimitReached,
            ));
        }

        if current_frame.depth > 0 {
            apply_execution_cost(
                heap,
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

    fn pre_kernel_execute<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        callee: &ResolvedActor,
        _nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        match &callee.identifier {
            FnIdentifier::Native(native_fn) => apply_execution_cost(
                heap,
                CostingReason::RunNative,
                |fee_table| fee_table.run_native_fn_cost(&native_fn),
                1,
            ),
            _ => Ok(()),
        }
    }

    fn pre_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            heap,
            CostingReason::CreateNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::CreateNode { size: 0 }),
            1,
        )?;
        Ok(())
    }

    fn post_drop_node(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            heap,
            CostingReason::DropNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode { size: 0 }),
            1,
        )?;

        apply_royalty_cost(heap, receiver, amount)?;
        Ok(())
    }

    fn on_lock_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            heap,
            CostingReason::LockSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::LockSubstate),
            1,
        )?;
        Ok(())
    }

    fn on_read_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            heap,
            CostingReason::ReadSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::ReadSubstate { size: size as u32 }),
            1,
        )?;
        Ok(())
    }

    fn on_write_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            heap,
            CostingReason::WriteSubstate,
            |fee_table| {
                fee_table.kernel_api_cost(CostingEntry::WriteSubstate { size: size as u32 })
            },
            1,
        )?;
        Ok(())
    }

    fn on_drop_lock(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            heap,
            CostingReason::DropLock,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
            1,
        )?;
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        code: &[u8],
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            heap,
            CostingReason::InstantiateWasm,
            |fee_table| fee_table.wasm_instantiation_per_byte(),
            code.len(),
        )
    }

    fn on_consume_cost_units(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        units: u32,
    ) -> Result<(), RuntimeError> {
        // We multiply by a large enough factor to ensure spin loops end within a fraction of a second.
        // These values will be tweaked, alongside the whole fee table.
        apply_execution_cost(heap, CostingReason::RunWasm, |_| units, 5)
    }
}
