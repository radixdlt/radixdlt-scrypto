use bitflags::bitflags;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
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

pub type FieldHandle = u32;

pub trait FieldPayloadMarker {}

impl<T: FieldPayloadMarker> FieldPayloadMarker for &T {}

/// System api to read/write fields
pub trait SystemFieldApi<E: Debug> {
    /// Retrieve the value of a field
    fn field_read(&mut self, handle: FieldHandle) -> Result<Vec<u8>, E>;

    /// Retrieve the value of a field
    fn field_read_typed<S: ScryptoDecode>(&mut self, handle: FieldHandle) -> Result<S, E> {
        let buf = self.field_read(handle)?;
        let typed_substate: S = scrypto_decode(&buf).map_err(|e| e).unwrap();
        Ok(typed_substate)
    }

    /// Write a value to a field
    fn field_write(&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), E>;

    /// Write a value to a field
    fn field_write_typed<S: ScryptoEncode>(
        &mut self,
        handle: FieldHandle,
        substate: &S,
    ) -> Result<(), E> {
        let buf = scrypto_encode(substate).unwrap();
        self.field_write(handle, buf)
    }

    /// Lock a field such that it becomes immutable
    fn field_lock(&mut self, handle: FieldHandle) -> Result<(), E>;

    /// Close a field handle so that it is no longer usable
    fn field_close(&mut self, handle: FieldHandle) -> Result<(), E>;
}
