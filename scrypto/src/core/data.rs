use radix_engine_lib::data::{scrypto_decode, scrypto_encode, ScryptoCustomTypeId};
use radix_engine_lib::engine::api::EngineApi;
use radix_engine_lib::engine::types::{
    ComponentOffset, KeyValueStoreOffset, LockHandle, RENodeId, SubstateOffset,
};
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::{Decode, Encode};
use scrypto::engine::scrypto_env::ScryptoEnv;

use crate::component::{ComponentStateSubstate, KeyValueStoreEntrySubstate};

pub struct DataRef<V: Encode<ScryptoCustomTypeId>> {
    lock_handle: LockHandle,
    value: V,
}

impl<V: fmt::Display + Encode<ScryptoCustomTypeId>> fmt::Display for DataRef<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode<ScryptoCustomTypeId>> DataRef<V> {
    pub fn new(lock_handle: LockHandle, value: V) -> DataRef<V> {
        DataRef { lock_handle, value }
    }
}

impl<V: Encode<ScryptoCustomTypeId>> Deref for DataRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode<ScryptoCustomTypeId>> Drop for DataRef<V> {
    fn drop(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls.sys_drop_lock(self.lock_handle).unwrap();
    }
}

pub struct DataRefMut<V: Encode<ScryptoCustomTypeId>> {
    lock_handle: LockHandle,
    offset: SubstateOffset,
    value: V,
}

impl<V: fmt::Display + Encode<ScryptoCustomTypeId>> fmt::Display for DataRefMut<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode<ScryptoCustomTypeId>> DataRefMut<V> {
    pub fn new(lock_handle: LockHandle, offset: SubstateOffset, value: V) -> DataRefMut<V> {
        DataRefMut {
            lock_handle,
            offset,
            value,
        }
    }
}

impl<V: Encode<ScryptoCustomTypeId>> Drop for DataRefMut<V> {
    fn drop(&mut self) {
        let mut syscalls = ScryptoEnv;
        let bytes = scrypto_encode(&self.value);
        let substate = match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                scrypto_encode(&KeyValueStoreEntrySubstate(Some(bytes)))
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                scrypto_encode(&ComponentStateSubstate { raw: bytes })
            }
            s @ _ => panic!("Unsupported substate: {:?}", s),
        };

        syscalls.sys_write(self.lock_handle, substate).unwrap();
        syscalls.sys_drop_lock(self.lock_handle).unwrap();
    }
}

impl<V: Encode<ScryptoCustomTypeId>> Deref for DataRefMut<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode<ScryptoCustomTypeId>> DerefMut for DataRefMut<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct DataPointer<V: 'static + Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>> {
    node_id: RENodeId,
    offset: SubstateOffset,
    phantom_data: PhantomData<V>,
}

impl<V: 'static + Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>> DataPointer<V> {
    pub fn new(node_id: RENodeId, offset: SubstateOffset) -> Self {
        Self {
            node_id,
            offset,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let mut syscalls = ScryptoEnv;

        let lock_handle = syscalls
            .sys_lock_substate(self.node_id, self.offset.clone(), false)
            .unwrap();
        let raw_substate = syscalls.sys_read(lock_handle).unwrap();
        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: substate,
                }
            }
        }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let mut syscalls = ScryptoEnv;

        let lock_handle = syscalls
            .sys_lock_substate(self.node_id, self.offset.clone(), true)
            .unwrap();
        let raw_substate = syscalls.sys_read(lock_handle).unwrap();

        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: substate,
                }
            }
        }
    }
}
