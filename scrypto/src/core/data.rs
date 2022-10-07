use crate::buffer::*;
use crate::component::{ComponentStateSubstate, KeyValueStoreEntrySubstate};
use crate::engine::api::RadixEngineInput;
use crate::engine::api::RadixEngineInput::SubstateWrite;
use crate::engine::call_engine;
use crate::engine::types::{ComponentOffset, RENodeId, SubstateId, SubstateOffset};
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::{Decode, Encode};

pub struct DataRef<V: Encode> {
    substate_id: SubstateId,
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRef<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRef<V> {
    pub fn new(substate_id: SubstateId, value: V) -> DataRef<V> {
        DataRef { substate_id, value }
    }
}

impl<V: Encode> Deref for DataRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode> Drop for DataRef<V> {
    fn drop(&mut self) {
        let input = RadixEngineInput::DropRef(self.substate_id.clone());
        let _: () = call_engine(input);
    }
}



pub struct DataRefMut<V: Encode> {
    substate_id: SubstateId,
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRefMut<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRefMut<V> {
    pub fn new(substate_id: SubstateId, value: V) -> DataRefMut<V> {
        DataRefMut { substate_id, value }
    }
}

impl<V: Encode> Drop for DataRefMut<V> {
    fn drop(&mut self) {
        let bytes = scrypto_encode(&self.value);
        let substate = match &self.substate_id {
            SubstateId(RENodeId::KeyValueStore(..), ..) => {
                scrypto_encode(&KeyValueStoreEntrySubstate(Some(bytes)))
            }
            SubstateId(
                RENodeId::Component(..),
                SubstateOffset::Component(ComponentOffset::State),
            ) => scrypto_encode(&ComponentStateSubstate { raw: bytes }),
            s @ _ => panic!("Unsupported substate: {:?}", s),
        };
        let input = SubstateWrite(self.substate_id.clone(), substate);
        let _: () = call_engine(input);

        let input = RadixEngineInput::DropRef(self.substate_id.clone());
        let _: () = call_engine(input);
    }
}

impl<V: Encode> Deref for DataRefMut<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode> DerefMut for DataRefMut<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct DataPointer<V: 'static + Encode + Decode> {
    substate_id: SubstateId,
    phantom_data: PhantomData<V>,
}

impl<V: 'static + Encode + Decode> DataPointer<V> {
    pub fn new(substate_id: SubstateId) -> Self {
        Self {
            substate_id,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let input = RadixEngineInput::CreateRef(self.substate_id.clone(), false);
        let _: () = call_engine(input);

        let input = RadixEngineInput::SubstateRead(self.substate_id.clone());
        match &self.substate_id {
            SubstateId(RENodeId::KeyValueStore(..), ..) => {
                let substate: KeyValueStoreEntrySubstate = call_engine(input);
                DataRef {
                    substate_id: self.substate_id.clone(),
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateId(_, SubstateOffset::Component(ComponentOffset::State)) => {
                let substate: ComponentStateSubstate = call_engine(input);
                DataRef {
                    substate_id: self.substate_id.clone(),
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = call_engine(input);
                DataRef {
                    substate_id: self.substate_id.clone(),
                    value: substate
                }
            }
        }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let input = RadixEngineInput::CreateRef(self.substate_id.clone(), true);
        let _: () = call_engine(input);

        let input = RadixEngineInput::SubstateRead(self.substate_id.clone());
        match &self.substate_id {
            SubstateId(RENodeId::KeyValueStore(..), ..) => {
                let substate: KeyValueStoreEntrySubstate = call_engine(input);
                DataRefMut {
                    substate_id: self.substate_id.clone(),
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateId(
                RENodeId::Component(..),
                SubstateOffset::Component(ComponentOffset::State),
            ) => {
                let substate: ComponentStateSubstate = call_engine(input);
                DataRefMut {
                    substate_id: self.substate_id.clone(),
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = call_engine(input);
                DataRefMut {
                    substate_id: self.substate_id.clone(),
                    value: substate
                }
            }
        }
    }

}
