#![cfg_attr(not(feature = "std"), no_std)]

use sbor::describe::*;
use scrypto::prelude::*;

#[derive(NonFungibleData, Debug, Eq, PartialEq)]
pub struct Sample {
    pub a: u32,
    #[scrypto(mutable)]
    pub b: String,
}

#[test]
fn test_non_fungible_data() {
    let instance = Sample {
        a: 1,
        b: "Test".to_owned(),
    };
    let instance_decoded =
        Sample::decode(&instance.immutable_data(), &instance.mutable_data()).unwrap();
    assert_eq!(instance_decoded, instance);

    let immutable_data_schema = Sample::immutable_data_schema();
    assert_eq!(
        immutable_data_schema,
        Type::Struct {
            name: "Sample".to_owned(),
            fields: Fields::Named {
                named: vec![("a".to_owned(), Type::U32)]
            },
        }
    );

    let mutable_data_schema = Sample::mutable_data_schema();
    assert_eq!(
        mutable_data_schema,
        Type::Struct {
            name: "Sample".to_owned(),
            fields: Fields::Named {
                named: vec![("b".to_owned(), Type::String)]
            },
        }
    );
}
