use crate::api::field_api::FieldHandle;
use crate::api::{ActorRefHandle, FieldIndex};
use crate::internal_prelude::*;
use crate::types::*;
use bitflags::bitflags;
use radix_engine_interface::api::{ActorStateHandle, LockFlags};
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

bitflags! {
    #[derive(Sbor)]
    pub struct EventFlags: u32 {
        /// With this flag on, an event will not be reverted if the transaction fails.
        const FORCE_WRITE = 0b00000001;
    }
}

/// Api which exposes methods in the context of the actor
pub trait SystemActorApi<E> {
    /// Retrieve the current blueprint id
    fn actor_get_blueprint_id(&mut self) -> Result<BlueprintId, E>;

    /// Retrieve the current method actor's node id
    fn actor_get_node_id(&mut self, ref_handle: ActorRefHandle) -> Result<NodeId, E>;

    /// Check if a feature is enabled for a given object
    fn actor_is_feature_enabled(
        &mut self,
        state_handle: ActorStateHandle,
        feature: &str,
    ) -> Result<bool, E>;

    /// Open a field in a given object for reading/writing
    fn actor_open_field(
        &mut self,
        state_handle: ActorStateHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<FieldHandle, E>;

    /// Emits an event of the current actor
    fn actor_emit_event(
        &mut self,
        event_name: String,
        event_data: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), E>;
}
