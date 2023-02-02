use crate::{
    errors::RuntimeError,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    kernel::{CallFrameUpdate, ResolvedActor},
};
use radix_engine_interface::api::types::RENodeId;

pub struct TransactionHashModule;

impl TransactionHashModule {
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
