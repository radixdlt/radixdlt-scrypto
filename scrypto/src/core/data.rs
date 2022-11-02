use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::{Decode, Encode};

use crate::buffer::*;
use crate::component::{ComponentStateSubstate, KeyValueStoreEntrySubstate};
use crate::data::*;
use crate::engine::{api::*, types::*, utils::*};

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
        let input = RadixEngineInput::DropLock(self.lock_handle);
        let _: () = call_engine(input);
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
        let input = RadixEngineInput::Write(self.lock_handle, substate);
        let _: () = call_engine(input);

        let input = RadixEngineInput::DropLock(self.lock_handle);
        let _: () = call_engine(input);
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
        let input = RadixEngineInput::LockSubstate(self.node_id, self.offset.clone(), false);
        let lock_handle: LockHandle = call_engine(input);

        let input = RadixEngineInput::Read(lock_handle);
        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = call_engine(input);
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = call_engine(input);
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = call_engine(input);
                DataRef {
                    lock_handle,
                    value: substate,
                }
            }
        }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let input = RadixEngineInput::LockSubstate(self.node_id, self.offset.clone(), true);
        let lock_handle: LockHandle = call_engine(input);

        let input = RadixEngineInput::Read(lock_handle);
        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = call_engine(input);
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = call_engine(input);
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = call_engine(input);
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: substate,
                }
            }
        }
    }
}
