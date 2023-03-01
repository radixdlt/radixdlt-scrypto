#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Debug, PartialEq, Sbor)]
pub struct TestStructNamed {
    #[allow(unused_variables)]
    #[sbor(skip)]
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, PartialEq, Sbor)]
pub struct TestStructUnnamed(#[sbor(skip)] u32, u32);

#[derive(Debug, PartialEq, Sbor)]
pub struct TestStructUnit;

#[derive(Debug, PartialEq, Sbor)]
pub enum TestEnum {
    A {
        #[sbor(skip)]
        x: u32,
        y: u32,
    },
    B(#[sbor(skip)] u32, u32),
    C,
}

#[test]
fn test_struct_with_skip() {
    let a = TestStructNamed { x: 1, y: 2 };
    let b = TestStructUnnamed(3, 4);
    let c = TestStructUnit;

    let mut bytes = Vec::with_capacity(512);
    let mut encoder = BasicEncoder::new(&mut bytes, 255);
    encoder.encode(&a).unwrap();
    encoder.encode(&b).unwrap();
    encoder.encode(&c).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        vec![
          33, // tuple type 
          1, // number of fields
          9, 2, 0, 0, 0, // field value

          33,  // tuple type 
          1,  // number of fields
          9, 4, 0, 0, 0,  // field value

          33, // tuple type
          0,  // number of fields
        ],
        bytes
    );

    let mut decoder = BasicDecoder::new(&bytes, 255);
    let a = decoder.decode::<TestStructNamed>().unwrap();
    let b = decoder.decode::<TestStructUnnamed>().unwrap();
    let c = decoder.decode::<TestStructUnit>().unwrap();

    assert_eq!(TestStructNamed { x: 0, y: 2 }, a);
    assert_eq!(TestStructUnnamed(0, 4), b);
    assert_eq!(TestStructUnit {}, c);
}

#[test]
fn test_enum_with_skip() {
    let a = TestEnum::A { x: 1, y: 2 };
    let b = TestEnum::B(3, 4);
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
            1,  // number of fields
            9, 2, 0, 0, 0, // field value

            34, // enum type
            1, // "B"
            1,  // number of fields
            9, 4, 0, 0, 0, // field value
            
            34, // enum type
            2, // "C"
            0,  // number of fields
        ],
        bytes
    );

    let mut decoder = BasicDecoder::new(&bytes, 255);
    let a = decoder.decode::<TestEnum>().unwrap();
    let b = decoder.decode::<TestEnum>().unwrap();
    let c = decoder.decode::<TestEnum>().unwrap();

    assert_eq!(TestEnum::A { x: 0, y: 2 }, a);
    assert_eq!(TestEnum::B(0, 4), b);
    assert_eq!(TestEnum::C, c);
}
