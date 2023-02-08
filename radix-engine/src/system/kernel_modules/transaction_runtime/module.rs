use crate::{
    blueprints::transaction_runtime::TransactionRuntimeSubstate,
    errors::RuntimeError,
    kernel::{CallFrameUpdate, ResolvedActor},
    kernel::{KernelModule, KernelModuleApi},
    system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use radix_engine_interface::crypto::Hash;
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TransactionRuntimeModule {
    pub tx_hash: Hash,
}

impl KernelModule for TransactionRuntimeModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let hash = api.get_module_state().transaction_runtime.tx_hash.clone();

        let node_id = api.allocate_node_id(RENodeType::TransactionRuntime)?;
        api.create_node(
            node_id,
            RENodeInit::TransactionRuntime(TransactionRuntimeSubstate {
                hash,
                next_id: 0u32,
                instruction_index: 0u32,
            }),
            BTreeMap::new(),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.drop_node(RENodeId::TransactionRuntime)?;

        Ok(())
    }

    fn before_new_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &ResolvedActor,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::TransactionRuntime);

        Ok(())
    }
}
