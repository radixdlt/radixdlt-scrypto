use crate::buffer::*;
use crate::core::DataAddress;
use crate::engine::api::RadixEngineInput::WriteData;
use crate::engine::call_engine;
use sbor::rust::cell::{Ref, RefMut};
use sbor::rust::ops::{Deref, DerefMut};
use sbor::Encode;

pub struct DataValueRef<V: Encode> {
    pub value: V,
}

impl<V: Encode> Deref for DataValueRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}


pub struct DataRef<'a, V: Encode> {
    pub value: Ref<'a, V>,
}

impl<'a, V: Encode> Deref for DataRef<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

pub struct DataRefMut<'a, V: Encode> {
    pub address: DataAddress,
    pub value: RefMut<'a, V>,
}

impl<'a, V: Encode> Drop for DataRefMut<'a, V> {
    fn drop(&mut self) {
        let bytes = scrypto_encode(self.value.deref());
        let input = WriteData(self.address.clone(), bytes);
        let _: () = call_engine(input);
    }
}

impl<'a, V: Encode> Deref for DataRefMut<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<'a, V: Encode> DerefMut for DataRefMut<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}
