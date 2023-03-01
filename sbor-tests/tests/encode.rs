#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Categorize, Encode)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Categorize, Encode)]
pub struct TestStructUnnamed(u32);

#[derive(Categorize, Encode)]
pub struct TestStructUnit;

#[derive(Categorize, Encode)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[derive(Categorize, Encode)]
pub enum EmptyEnum {}

#[test]
fn test_encode_struct() {
    let a = TestStructNamed { state: 3 };
    let b = TestStructUnnamed(3);
    let c = TestStructUnit {};

    let mut bytes = Vec::with_capacity(512);
    let mut encoder = BasicEncoder::new(&mut bytes, 255);
    encoder.encode(&a).unwrap();
    encoder.encode(&b).unwrap();
    encoder.encode(&c).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            33, // tuple type
            1,  // number of fields
            9, 3, 0, 0, 0, // field value
            
            33, // tuple type
            1,  // number of fields
            9, 3, 0, 0, 0, // field value
            
            33, // tuple type
            0,  // number of fields
        ],
        bytes
    );
}

#[test]
fn test_encode_enum() {
    let a = TestEnum::A { x: 2, y: 3 };
    let b = TestEnum::B(1);
    let c = TestEnum::C;

    let mut bytes = Vec::with_capacity(512);
    let mut encoder = BasicEncoder::new(&mut bytes, 255);
    encoder.encode(&a).unwrap();
    encoder.encode(&b).unwrap();
    encoder.encode(&c).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            34, // enum type
            0, // "A"
            2,  // number of fields
            9, 2, 0, 0, 0, // field value
            9, 3, 0, 0, 0,  // field value

            34, // enum type
            1, // "B"
            1,  // number of fields
            9, 1, 0, 0, 0, // field value
            
            34, // enum type
            2, // "C"
            0,  // number of fields
        ],
        bytes
    );
}
