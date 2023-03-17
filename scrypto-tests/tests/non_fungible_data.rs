#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
struct Sample {
    a: u32,
    #[mutable]
    b: String,
}

#[test]
fn test_non_fungible_data() {
    let mutable_fields = Sample::MUTABLE_FIELDS;
    assert_eq!(mutable_fields, ["b"]);
}
