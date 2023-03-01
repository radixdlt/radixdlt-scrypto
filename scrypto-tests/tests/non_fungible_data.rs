#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[derive(NonFungibleData)]
struct A {
    a: u32,
    #[mutable]
    b: u8,
}
