use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::ClientFieldLockApi;
use radix_engine_interface::api::{ClientActorApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode, ScryptoValue,
};
use radix_engine_interface::types::*;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::rust::prelude::*;
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
        env.field_lock_release(self.lock_handle).unwrap();
    }
}

pub enum OriginalData {
    KeyValueStoreEntry(ScryptoValue),
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
            OriginalData::KeyValueStoreEntry(_) => scrypto_encode(&Some(&self.value)).unwrap(),
            OriginalData::ComponentAppState(_) => scrypto_encode(&self.value).unwrap(),
        };
        env.field_lock_write(self.lock_handle, substate).unwrap();
        env.field_lock_release(self.lock_handle).unwrap();
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

pub struct ComponentStatePointer<V: 'static + ScryptoEncode + ScryptoDecode> {
    phantom_data: PhantomData<V>,
}

impl<V: 'static + ScryptoEncode + ScryptoDecode> ComponentStatePointer<V> {
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .actor_open_field(
                OBJECT_HANDLE_SELF,
                ComponentField::State0 as u8,
                LockFlags::read_only(),
            )
            .unwrap();
        let raw_substate = env.field_lock_read(lock_handle).unwrap();
        let value: V = scrypto_decode(&raw_substate).unwrap();
        DataRef { lock_handle, value }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .actor_open_field(
                OBJECT_HANDLE_SELF,
                ComponentField::State0 as u8,
                LockFlags::MUTABLE,
            )
            .unwrap();
        let raw_substate = env.field_lock_read(lock_handle).unwrap();
        let value: V = scrypto_decode(&raw_substate).unwrap();
        DataRefMut {
            lock_handle,
            original_data: OriginalData::ComponentAppState(raw_substate),
            value,
        }
    }
}
