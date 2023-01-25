use crate::api::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub trait ClientSubstateApi<E: Debug> {
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, E>;
    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, E>;
    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
}
