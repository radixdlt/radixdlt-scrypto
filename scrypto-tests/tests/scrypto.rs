#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[scrypto(Encode, Decode, Categorize, NonFungibleData, Describe)]
pub struct TestStruct {
    pub a: u32,
    #[scrypto(skip)]
    #[sbor(skip)]
    pub b: String,
}

#[scrypto(Encode, Decode, Categorize, Describe)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8),
    C,
}
