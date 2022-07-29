use crate::buffer::*;
use crate::core::DataAddress;
use crate::engine::api::RadixEngineInput::WriteData;
use crate::engine::call_engine;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::{Decode, Encode};

pub struct DataRef<V: Encode> {
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRef<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRef<V> {
    pub fn new(value: V) -> DataRef<V> {
        DataRef { value }
    }
}

impl<V: Encode> Deref for DataRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct DataRefMut<V: Encode> {
    address: DataAddress,
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRefMut<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRefMut<V> {
    pub fn new(address: DataAddress, value: V) -> DataRefMut<V> {
        DataRefMut { address, value }
    }
}

impl<V: Encode> Drop for DataRefMut<V> {
    fn drop(&mut self) {
        let bytes = scrypto_encode(&self.value);
        let input = WriteData(self.address.clone(), bytes);
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
    address: DataAddress,
    phantom_data: PhantomData<V>,
}

impl<V: 'static + Encode + Decode> DataPointer<V> {
    pub fn new(address: DataAddress) -> Self {
        Self {
            address,
            phantom_data: PhantomData,
        }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let input = ::scrypto::engine::api::RadixEngineInput::ReadData(self.address.clone());
        let value: V = call_engine(input);
        DataRefMut {
            address: self.address.clone(),
            value,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let input = ::scrypto::engine::api::RadixEngineInput::ReadData(self.address.clone());
        let value: V = call_engine(input);
        DataRef { value }
    }
}
