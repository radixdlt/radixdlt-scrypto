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
    /*
    let instance = Sample {
        a: 1,
        b: "Test".to_owned(),
    };
    let instance_decoded = Sample::decode(
        &instance.immutable_data().unwrap(),
        &instance.mutable_data().unwrap(),
    )
    .unwrap();
    assert_eq!(instance_decoded, instance);
     */
}
