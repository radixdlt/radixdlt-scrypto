#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[derive(ScryptoData)]
pub struct TestStruct {
    pub a: u32,
    pub b: String,
}

#[derive(ScryptoData)]
pub struct TestEnum<T: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId> + TypeId<ScryptoCustomTypeId>> {
    A(u32, T),
    B {
        named: String
    },
    C
}
