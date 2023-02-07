use crate::{
    blueprints::transaction_runtime::TransactionRuntimeSubstate,
    errors::RuntimeError,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    kernel::{CallFrameUpdate, ResolvedActor},
    system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use radix_engine_interface::crypto::Hash;
use sbor::rust::collections::BTreeMap;

pub struct TransactionRuntimeModule;

impl TransactionRuntimeModule {
    pub fn initialize<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        tx_hash: Hash,
    ) -> Result<(), RuntimeError> {
        let node_id = api.allocate_node_id(RENodeType::TransactionRuntime)?;
        api.create_node(
            node_id,
            RENodeInit::TransactionRuntime(TransactionRuntimeSubstate {
                hash: tx_hash,
                next_id: 0u32,
                instruction_index: 0u32,
            }),
            BTreeMap::new(),
        )?;
        Ok(())
    }

    pub fn teardown<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<TransactionRuntimeSubstate, RuntimeError> {
        let substate: TransactionRuntimeSubstate =
            api.drop_node(RENodeId::TransactionRuntime)?.into();

        Ok(substate)
    }

    pub fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if api
            .get_visible_node_data(RENodeId::TransactionRuntime)
            .is_ok()
        {
            call_frame_update
                .node_refs_to_copy
                .insert(RENodeId::TransactionRuntime);
        }

        Ok(())
    }
}
