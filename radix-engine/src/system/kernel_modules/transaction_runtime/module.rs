use crate::{
    blueprints::transaction_runtime::TransactionRuntimeSubstate,
    errors::RuntimeError,
    kernel::{
        kernel_api::KernelSubstateApi, KernelModule, KernelModuleId, KernelModuleState,
        KernelNodeApi,
    },
    kernel::{CallFrameUpdate, ResolvedActor},
    system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeModule {
    pub tx_hash: Hash,
}

impl KernelModuleState for TransactionRuntimeModule {
    const ID: u8 = KernelModuleId::TransactionRuntime as u8;
}

impl KernelModule for TransactionRuntimeModule {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        if let Some(state) = api.get_module_state::<TransactionRuntimeModule>() {
            let hash = state.tx_hash.clone();

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
        }
        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state::<TransactionRuntimeModule>().is_none() {
            return Ok(());
        }

        api.drop_node(RENodeId::TransactionRuntime)?;

        Ok(())
    }

    fn before_create_frame<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _actor: &ResolvedActor,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state::<TransactionRuntimeModule>().is_none() {
            return Ok(());
        }

        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::TransactionRuntime);

        Ok(())
    }
}
