#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::Decode;
use sbor::Decoder;
use sbor::TypeId;

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

#[test]
fn test_decode_struct() {
    #[rustfmt::skip]
    let bytes = vec![
        16, // struct type
        18, // fields type
        1, 0, 0, 0, // number of fields
        9, 3, 0, 0, 0, // field value
        
        16,  // struct type
        19,  // fields type
        1, 0, 0, 0,  // number of fields
        9, 3, 0, 0, 0,  // field value
        
        16, // struct type
        20 // fields type
    ];

    let mut decoder = Decoder::with_type(&bytes);
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
        17, // enum type
        0, // enum index
        18, // fields type
        2, 0, 0, 0,  // number of fields
        9, 2, 0, 0, 0, // field value
        9, 3, 0, 0, 0,  // field value

        17, // enum type
        1,  // enum index
        19, // fields type
        1, 0, 0, 0, // number of fields
        9, 1, 0, 0, 0, // field value
        
        17, // enum type
        2,  // enum index
        20  // fields type
    ];

    let mut decoder = Decoder::with_type(&bytes);
    let a = TestEnum::decode(&mut decoder).unwrap();
    let b = TestEnum::decode(&mut decoder).unwrap();
    let c = TestEnum::decode(&mut decoder).unwrap();

    assert_eq!(TestEnum::A { x: 2, y: 3 }, a);
    assert_eq!(TestEnum::B(1), b);
    assert_eq!(TestEnum::C, c);
}
