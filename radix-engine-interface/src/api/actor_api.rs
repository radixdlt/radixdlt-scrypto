use crate::types::*;
use radix_engine_interface::api::LockFlags;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    /// Lock a field in the current object actor for reading/writing
    fn lock_field(&mut self, field: u8, flags: LockFlags) -> Result<LockHandle, E>;

    // TODO: Should this be exposed as a virtual field instead?
    /// Lock a field in the current object actor's parent for reading/writing
    fn lock_parent_field(&mut self, field: u8, flags: LockFlags) -> Result<LockHandle, E>;

    // TODO: Add specific object read/write lock apis

    fn actor_lock_key_value_entry(
        &mut self,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<LockHandle, E> {
        self.actor_lock_key_value_handle_entry(0u8, key, flags)
    }

    fn actor_lock_key_value_handle_entry(
        &mut self,
        kv_handle: u8,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<LockHandle, E>;

    fn actor_key_value_entry_remove(&mut self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, E>;

    fn get_info(&mut self) -> Result<ObjectInfo, E>;

    fn get_global_address(&mut self) -> Result<GlobalAddress, E>;

    fn get_blueprint(&mut self) -> Result<Blueprint, E>;
}
