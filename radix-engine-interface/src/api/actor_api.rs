use crate::api::FieldIndex;
use crate::types::*;
use radix_engine_interface::api::{LockFlags, ObjectHandle};
use sbor::rust::fmt::Debug;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    /// Lock a field in the current object actor for reading/writing
    fn actor_lock_field(
        &mut self,
        object_handle: ObjectHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<LockHandle, E>;

    // TODO: Remove
    fn actor_get_info(&mut self) -> Result<ObjectInfo, E>;

    fn actor_get_node_id(&mut self) -> Result<NodeId, E>;

    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, E>;

    fn actor_get_blueprint(&mut self) -> Result<Blueprint, E>;
}
