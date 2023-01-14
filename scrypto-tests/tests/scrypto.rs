#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct TestStruct {
    pub a: u32,
    #[scrypto(skip)]
    #[sbor(skip)]
    pub b: String,
}

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8),
    C,
}
