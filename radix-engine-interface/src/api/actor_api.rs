use crate::api::field_api::FieldHandle;
use crate::api::{FieldIndex, ObjectModuleId};
use crate::types::*;
use radix_engine_interface::api::{LockFlags, ObjectHandle};
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto_schema::BlueprintFeature;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    /// Retrieve the current blueprint id
    fn actor_get_blueprint_id(&mut self) -> Result<BlueprintId, E>;

    /// Open a field in a given object for reading/writing
    fn actor_open_field(
        &mut self,
        object_handle: ObjectHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<FieldHandle, E>;

    /// Check if a feature is enabled for a given object
    fn actor_is_feature_enabled(
        &mut self,
        object_handle: ObjectHandle,
        feature: impl BlueprintFeature,
    ) -> Result<bool, E>;

    /// Retrieve the current method actor's node id
    fn actor_get_node_id(&mut self) -> Result<NodeId, E>;

    /// Retrieve the current method actor's outer object
    fn actor_get_outer_object(&mut self) -> Result<GlobalAddress, E>;

    /// Retrieve the current method actor's global address
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, E>;

    /// Call a method on a module of the current method actor
    fn actor_call_module(
        &mut self,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
