#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::Decode;
use sbor::Decoder;

#[derive(Decode, Debug, PartialEq)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Decode, Debug, PartialEq)]
pub struct TestStructUnnamed(u32);

#[derive(Decode, Debug, PartialEq)]
pub struct TestStructUnit;

#[derive(Decode, Debug, PartialEq)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[test]
fn test_decode_struct() {
    #[rustfmt::skip]
    let bytes = vec![
        20, // struct type
        22, // fields type
        1, 0, 0, 0, // number of fields
        5, 0, 0, 0, 115, 116, 97, 116, 101, // field name
        9, 3, 0, 0, 0, // field value
        
        20,  // struct type
        23,  // fields type
        1, 0, 0, 0,  // number of fields
        9, 3, 0, 0, 0,  // field value
        
        20, // struct type
        24 // fields type
    ];

    let mut decoder = Decoder::with_metadata(&bytes);
    let a = TestStructNamed::decode(&mut decoder).unwrap();
    let b = TestStructUnnamed::decode(&mut decoder).unwrap();
    let c = TestStructUnit::decode(&mut decoder).unwrap();

    assert_eq!(TestStructNamed { state: 3 }, a);
    assert_eq!(TestStructUnnamed(3), b);
    assert_eq!(TestStructUnit {}, c);
}

#[test]
fn test_decode_enum() {
    #[rustfmt::skip]
    let bytes = vec![
        21, // enum type
        0, // enum index
        1, 0, 0, 0, 65, // variant name
        22, // fields type
        2, 0, 0, 0,  // number of fields
        1, 0, 0, 0, 120, // field name
        9, 2, 0, 0, 0, // field value
        1, 0, 0, 0, 121,  // field name
        9, 3, 0, 0, 0,  // field value

        21, // enum type
        1,  // enum index
        1, 0, 0, 0, 66, // variant name
        23, // fields type
        1, 0, 0, 0, // number of fields
        9, 1, 0, 0, 0, // field value
        
        21, // enum type
        2,  // enum index
        1, 0, 0, 0, 67, // variant name
        24  // fields type
    ];

    let mut decoder = Decoder::with_metadata(&bytes);
    let a = TestEnum::decode(&mut decoder).unwrap();
    let b = TestEnum::decode(&mut decoder).unwrap();
    let c = TestEnum::decode(&mut decoder).unwrap();

    assert_eq!(TestEnum::A { x: 2, y: 3 }, a);
    assert_eq!(TestEnum::B(1), b);
    assert_eq!(TestEnum::C, c);
}
