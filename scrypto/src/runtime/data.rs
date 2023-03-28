use radix_engine_interface::api::component::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientSubstateApi;
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
        env.sys_drop_lock(self.lock_handle).unwrap();
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
            OriginalData::KeyValueStoreEntry(_) => {
                let substate: Option<ScryptoValue> =
                    Option::Some(scrypto_decode(&scrypto_encode(&self.value).unwrap()).unwrap());
                scrypto_encode(&substate).unwrap()
            }
            OriginalData::ComponentAppState(_) => scrypto_encode(&ComponentStateSubstate(
                scrypto_decode(&scrypto_encode(&self.value).unwrap()).unwrap(),
            ))
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

pub struct ComponentStatePointer<V: 'static + ScryptoEncode + ScryptoDecode> {
    node_id: NodeId,
    phantom_data: PhantomData<V>,
}

impl<V: 'static + ScryptoEncode + ScryptoDecode> ComponentStatePointer<V> {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .sys_lock_substate(
                &self.node_id,
                &SubstateKey::from_vec(vec![ComponentOffset::State0 as u8]).unwrap(),
                LockFlags::read_only(),
            )
            .unwrap();
        let raw_substate = env.sys_read_substate(lock_handle).unwrap();
        let value: V = scrypto_decode(&raw_substate).unwrap();
        DataRef { lock_handle, value }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .sys_lock_substate(
                &self.node_id,
                &SubstateKey::from_vec(vec![ComponentOffset::State0 as u8]).unwrap(),
                LockFlags::MUTABLE,
            )
            .unwrap();
        let raw_substate = env.sys_read_substate(lock_handle).unwrap();
        let value: V = scrypto_decode(&raw_substate).unwrap();
        DataRefMut {
            lock_handle,
            original_data: OriginalData::ComponentAppState(raw_substate),
            value,
        }
    }
}
