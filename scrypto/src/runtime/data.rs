use radix_engine_interface::api::component::*;
use radix_engine_interface::api::types::LockHandle;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::kv_store::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoEncode, ScryptoValue};
use sbor::rust::fmt;
use sbor::rust::ops::{Deref, DerefMut};
use scrypto::engine::scrypto_env::ScryptoEnv;

pub struct DataRef<V: ScryptoEncode> {
    lock_handle: LockHandle,
    value: V,
}

impl<V: fmt::Display + ScryptoEncode> fmt::Display for DataRef<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: ScryptoEncode> DataRef<V> {
    pub fn new(lock_handle: LockHandle, substate: V) -> DataRef<V> {
        DataRef {
            lock_handle,
            value: substate,
        }
    }
}

impl<V: ScryptoEncode> Deref for DataRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: ScryptoEncode> Drop for DataRef<V> {
    fn drop(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_drop_lock(self.lock_handle).unwrap();
    }
}

pub enum OriginalData {
    KeyValueStoreEntry(ScryptoValue, ScryptoValue),
    ComponentAppState(Vec<u8>),
}

pub struct DataRefMut<V: ScryptoEncode> {
    lock_handle: LockHandle,
    original_data: OriginalData,
    value: V,
}

impl<V: fmt::Display + ScryptoEncode> fmt::Display for DataRefMut<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: ScryptoEncode> DataRefMut<V> {
    pub fn new(lock_handle: LockHandle, original_data: OriginalData, value: V) -> DataRefMut<V> {
        DataRefMut {
            lock_handle,
            original_data,
            value,
        }
    }
}

impl<V: ScryptoEncode> Drop for DataRefMut<V> {
    fn drop(&mut self) {
        let mut env = ScryptoEnv;
        let substate = match &self.original_data {
            OriginalData::KeyValueStoreEntry(k, _) => {
                scrypto_encode(&KeyValueStoreEntrySubstate::Some(
                    k.clone(),
                    scrypto_decode(&scrypto_encode(&self.value).unwrap()).unwrap(),
                ))
                .unwrap()
            }
            OriginalData::ComponentAppState(_) => scrypto_encode(&ComponentStateSubstate {
                raw: scrypto_encode(&self.value).unwrap(),
            })
            .unwrap(),
        };
        env.sys_write_substate(self.lock_handle, substate).unwrap();
        env.sys_drop_lock(self.lock_handle).unwrap();
    }
}

impl<V: ScryptoEncode> Deref for DataRefMut<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: ScryptoEncode> DerefMut for DataRefMut<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
