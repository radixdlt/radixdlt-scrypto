use bitflags::bitflags;
use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
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

pub type FieldLockHandle = u32;

/// A high level api to read/write fields
pub trait ClientFieldLockApi<E: Debug> {
    fn field_lock_read(&mut self, handle: FieldLockHandle) -> Result<Vec<u8>, E>;

    fn field_lock_read_typed<S: ScryptoDecode>(&mut self, handle: FieldLockHandle) -> Result<S, E> {
        let buf = self.field_lock_read(handle)?;
        let typed_substate: S = scrypto_decode(&buf).unwrap();
        Ok(typed_substate)
    }

    fn field_lock_write(&mut self, handle: FieldLockHandle, buffer: Vec<u8>) -> Result<(), E>;

    fn field_lock_write_typed<S: ScryptoEncode>(
        &mut self,
        handle: FieldLockHandle,
        substate: S,
    ) -> Result<(), E> {
        let buf = scrypto_encode(&substate).unwrap();
        self.field_lock_write(handle, buf)
    }

    fn field_lock_release(&mut self, handle: FieldLockHandle) -> Result<(), E>;
}
