use bitflags::bitflags;
use radix_common::data::scrypto::*;
use sbor::rust::fmt::Debug;
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
    fn field_read(&mut self, handle: FieldHandle) -> Result<ScryptoOwnedRawValue, E>;

    /// Retrieve the value of a field
    fn field_read_typed<S: ScryptoDecode>(&mut self, handle: FieldHandle) -> Result<S, E> {
        let value = self.field_read(handle)?;
        Ok(value.decode_as().unwrap())
    }

    /// Write a value to a field
    fn field_write(
        &mut self,
        handle: FieldHandle,
        buffer: ScryptoUnvalidatedRawValue,
    ) -> Result<(), E>;

    /// Write a value to a field
    fn field_write_typed<S: ScryptoEncode>(
        &mut self,
        handle: FieldHandle,
        substate: &S,
    ) -> Result<(), E> {
        let value = scrypto_encode_to_value(substate).unwrap();
        self.field_write(handle, value.as_unvalidated())
    }

    /// Lock a field such that it becomes immutable
    fn field_lock(&mut self, handle: FieldHandle) -> Result<(), E>;

    /// Close a field handle so that it is no longer usable
    fn field_close(&mut self, handle: FieldHandle) -> Result<(), E>;
}
