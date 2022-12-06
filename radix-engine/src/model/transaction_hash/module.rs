use crate::engine::{CallFrameUpdate, REActor, RuntimeError, SystemApi};
use radix_engine_interface::api::types::RENodeId;

pub struct TransactionHashModule;

impl TransactionHashModule {
    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &REActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = api.get_visible_node_ids()?;
        let transaction_hash_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::TransactionHash(..)))
            .unwrap();
        call_frame_update
            .node_refs_to_copy
            .insert(transaction_hash_id);

        Ok(())
    }
}
