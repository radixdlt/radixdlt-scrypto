use radix_engine_interface::api::types::{GlobalAddress, RENodeId};
use radix_engine_interface::constants::ENTITY_OWNER_TOKEN;
use crate::engine::{CallFrameUpdate, REActor, RuntimeError, SystemApi};

pub struct EntityModule;

impl EntityModule {
    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &REActor,
        _system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        call_frame_update.node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(ENTITY_OWNER_TOKEN)));
        Ok(())
    }
}
