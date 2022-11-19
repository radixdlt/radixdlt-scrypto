#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::*;

#[derive(TypeId, Decode, Debug, PartialEq)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(TypeId, Decode, Debug, PartialEq)]
pub struct TestStructUnnamed(u32);

#[derive(TypeId, Decode, Debug, PartialEq)]
pub struct TestStructUnit;

#[derive(TypeId, Decode, Debug, PartialEq)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[derive(TypeId, Decode, Debug, PartialEq)]
pub enum EmptyEnum {}

#[test]
fn test_decode_struct() {
    #[rustfmt::skip]
    let bytes = vec![
        16, // struct type
        1, 0, 0, 0, // number of fields
        9, 3, 0, 0, 0, // field value
        
        16,  // struct type
        1, 0, 0, 0,  // number of fields
        9, 3, 0, 0, 0,  // field value
        
        16, // struct type
        0, 0, 0, 0,  // number of fields
    ];

    let mut decoder = VecDecoder::<NoCustomTypeId>::new(&bytes);
    let a = decoder.decode::<TestStructNamed>().unwrap();
    let b = decoder.decode::<TestStructUnnamed>().unwrap();
    let c = decoder.decode::<TestStructUnit>().unwrap();

    assert_eq!(TestStructNamed { state: 3 }, a);
    assert_eq!(TestStructUnnamed(3), b);
    assert_eq!(TestStructUnit {}, c);
}

#[test]
fn test_decode_enum() {
    #[rustfmt::skip]
    let bytes = vec![
        17, // enum type
        1, 0, 0, 0, // string size
        65, // "A"
        2, 0, 0, 0,  // number of fields
        9, 2, 0, 0, 0, // field value
        9, 3, 0, 0, 0,  // field value

        17, // enum type
        1, 0, 0, 0, // string size
        66, // "B"
        1, 0, 0, 0, // number of fields
        9, 1, 0, 0, 0, // field value
        
        17, // enum type
        1, 0, 0, 0, // string size
        67, // "C"
        0, 0, 0, 0,  // number of fields
    ];

    let mut decoder = VecDecoder::<NoCustomTypeId>::new(&bytes);
    let a = decoder.decode::<TestEnum>().unwrap();
    let b = decoder.decode::<TestEnum>().unwrap();
    let c = decoder.decode::<TestEnum>().unwrap();

    assert_eq!(TestEnum::A { x: 2, y: 3 }, a);
    assert_eq!(TestEnum::B(1), b);
    assert_eq!(TestEnum::C, c);
}

#[test]
fn test_decode_empty_enum() {
    #[rustfmt::skip]
    let bytes = vec![
        17, // enum type
        1, 0, 0, 0, // string size
        65, // "A"
        2, 0, 0, 0,  // number of fields
        9, 2, 0, 0, 0, // field value
        9, 3, 0, 0, 0,  // field value
    ];

    let mut decoder = VecDecoder::<NoCustomTypeId>::new(&bytes);
    let result = decoder.decode::<EmptyEnum>();

    assert!(matches!(result, Err(DecodeError::UnknownDiscriminator(_))));
}
