use radix_engine_interface::api::LockFlags;
use crate::types::*;
use sbor::rust::fmt::Debug;

/// Api which exposes methods in the context of the actor
pub trait ClientActorApi<E: Debug> {
    /// Lock a field in the current object actor for reading/writing
    fn lock_field(&mut self, field: u8, flags: LockFlags) -> Result<LockHandle, E>;

    // TODO: Add specific object read/write lock apis

    fn get_global_address(&mut self) -> Result<GlobalAddress, E>;
    fn get_blueprint(&mut self) -> Result<Blueprint, E>;
}
