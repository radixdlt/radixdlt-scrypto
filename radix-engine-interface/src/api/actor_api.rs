use crate::api::field_api::FieldHandle;
use crate::api::{FieldIndex, ObjectModuleId};
use crate::types::*;
use radix_engine_interface::api::{LockFlags, ObjectHandle};
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    fn actor_get_blueprint(&mut self) -> Result<BlueprintId, E>;

    /// Open a field in the current method actor for reading/writing
    fn method_actor_open_field(
        &mut self,
        object_handle: ObjectHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<FieldHandle, E>;

    fn method_actor_is_feature_enabled(
        &mut self,
        object_handle: ObjectHandle,
        feature: &str,
    ) -> Result<bool, E>;

    fn method_actor_get_node_id(&mut self) -> Result<NodeId, E>;

    fn method_actor_get_outer_object(&mut self) -> Result<GlobalAddress, E>;

    fn method_actor_get_global_address(&mut self) -> Result<GlobalAddress, E>;

    fn method_actor_call_module(
        &mut self,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
