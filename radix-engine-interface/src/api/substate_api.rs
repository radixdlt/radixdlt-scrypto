use crate::api::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use bitflags::bitflags;
use sbor::*;

bitflags! {
    #[derive(Sbor)]
    pub struct LockFlags: u32 {
        /// Allows the locked substate to be mutated
        const MUTABLE = 0b00000001;
        /// Checks that the substate locked is unmodified from the beginning of
        /// the transaction. This is used mainly for locking fees in vaults which
        /// requires this in order to be able to support rollbacks
        const UNMODIFIED_BASE = 0b00000010;
        /// Forces a write of a substate even on a transaction failure
        /// Currently used for vault fees.
        const FORCE_WRITE = 0b00000100;
    }
}

impl LockFlags {
    pub fn read_only() -> Self {
        LockFlags::empty()
    }
}

pub trait ClientSubstateApi<E: Debug> {
    // TODO: expose non-SELF?
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, E>;
    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, E>;
    fn sys_write_substate(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
}
