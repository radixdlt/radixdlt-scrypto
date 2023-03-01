#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::*;

#[derive(Categorize, Decode, Debug, PartialEq)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Categorize, Decode, Debug, PartialEq)]
pub struct TestStructUnnamed(u32);

#[derive(Categorize, Decode, Debug, PartialEq)]
pub struct TestStructUnit;

#[derive(Categorize, Decode, Debug, PartialEq)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[derive(Categorize, Decode, Debug, PartialEq)]
pub enum EmptyEnum {}

#[test]
fn test_decode_struct() {
    #[rustfmt::skip]
    let bytes = vec![
        33, // tuple type
        1,  // number of fields
        9, 3, 0, 0, 0, // field value
        
        33, // tuple type
        1,  // number of fields
        9, 3, 0, 0, 0, // field value
        
        33, // tuple type
        0,  // number of fields
    ];

    let mut decoder = BasicDecoder::new(&bytes, 255);
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
        34, // enum type
        0, // "A"
        2,  // number of fields
        9, 2, 0, 0, 0, // field value
        9, 3, 0, 0, 0,  // field value

        34, // enum type
        1,  // "B"
        1,  // number of fields
        9, 1, 0, 0, 0, // field value
        
        34, // enum type
        2,  // "C"
        0,  // number of fields
    ];

    let mut decoder = BasicDecoder::new(&bytes, 255);
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
        34, // enum type
        1,  // string size
        65, // "A"
        2,  // number of fields
        9, 2, 0, 0, 0, // field value
        9, 3, 0, 0, 0, // field value
    ];

    let mut decoder = BasicDecoder::new(&bytes, 255);
    let result = decoder.decode::<EmptyEnum>();

    assert!(matches!(result, Err(DecodeError::UnknownDiscriminator(_))));
}
