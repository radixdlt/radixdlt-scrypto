use radix_common::prelude::*;

pub type KeyValueEntryHandle = u32;

pub trait SystemKeyValueEntryApi<E> {
    /// Reads the value of a key value entry
    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Option<ScryptoOwnedRawValue>, E>;

    /// Reads the value of a key value entry and decodes it into a specific type
    fn key_value_entry_get_typed<S: ScryptoDecode>(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Option<S>, E> {
        let buffer = self.key_value_entry_get(handle)?;
        let value: Option<S> = buffer.map(|value| value.decode_as().unwrap());
        Ok(value)
    }

    /// Set the value of a key value entry
    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryHandle,
        buffer: ScryptoUnvalidatedRawValue,
    ) -> Result<(), E>;

    /// Set the value of a key value entry
    fn key_value_entry_set_typed<S: ScryptoEncode>(
        &mut self,
        handle: KeyValueEntryHandle,
        value: S,
    ) -> Result<(), E> {
        let buffer = scrypto_encode_to_value(&value).unwrap();
        self.key_value_entry_set(handle, buffer.into_unvalidated())
    }

    /// Remove the value of a key value entry
    fn key_value_entry_remove(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Option<ScryptoOwnedRawValue>, E>;

    /// Lock the value of a key value entry making the value immutable
    fn key_value_entry_lock(&mut self, handle: KeyValueEntryHandle) -> Result<(), E>;

    /// Close the handle into the key value entry rendering it unusable after close
    fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) -> Result<(), E>;
}
