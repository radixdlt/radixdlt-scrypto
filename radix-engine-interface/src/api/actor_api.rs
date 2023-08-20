use crate::api::field_api::FieldHandle;
use crate::api::{ActorRefHandle, FieldIndex};
use crate::types::*;
use radix_engine_interface::api::{LockFlags, ActorStateHandle};
use sbor::rust::fmt::Debug;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    /// Retrieve the current blueprint id
    fn actor_get_blueprint_id(
        &mut self,
    ) -> Result<BlueprintId, E>;

    /// Open a field in a given object for reading/writing
    fn actor_open_field(
        &mut self,
        state_handle: ActorStateHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<FieldHandle, E>;

    /// Check if a feature is enabled for a given object
    fn actor_is_feature_enabled(
        &mut self,
        ref_handle: ActorStateHandle,
        feature: &str,
    ) -> Result<bool, E>;

    /// Retrieve the current method actor's node id
    fn actor_get_node_id(&mut self, ref_handle: ActorRefHandle) -> Result<NodeId, E>;

    /// Retrieve the current method actor's outer object
    fn actor_get_outer_object(&mut self) -> Result<GlobalAddress, E>;

    /// Retrieve the current method actor's global address
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, E>;
}
