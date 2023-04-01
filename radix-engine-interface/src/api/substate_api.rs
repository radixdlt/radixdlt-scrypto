use crate::types::*;
use bitflags::bitflags;
use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
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
        node_id: &NodeId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, E>;

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, E>;

    fn sys_read_substate_typed<S: ScryptoDecode>(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<S, E> {
        let buf = self.sys_read_substate(lock_handle)?;
        let typed_substate: S = scrypto_decode(&buf).unwrap();
        Ok(typed_substate)
    }

    fn sys_write_substate(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;

    fn sys_write_substate_typed<S: ScryptoEncode>(
        &mut self,
        lock_handle: LockHandle,
        substate: S,
    ) -> Result<(), E> {
        let buf = scrypto_encode(&substate).unwrap();
        self.sys_write_substate(lock_handle, buf)
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
}
