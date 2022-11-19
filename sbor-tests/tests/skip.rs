#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
pub struct TestStructNamed {
    #[allow(unused_variables)]
    #[sbor(skip)]
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
pub struct TestStructUnnamed(#[sbor(skip)] u32, u32);

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
pub struct TestStructUnit;

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
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
    let mut encoder = Encoder::<NoCustomTypeId>::new(&mut bytes);
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);

    #[rustfmt::skip]
    assert_eq!(
        vec![
          16, // struct type 
          1, // number of fields
          9, 2, 0, 0, 0, // field value

          16,  // struct type 
          1,  // number of fields
          9, 4, 0, 0, 0,  // field value

          16, // struct type
          0,  // number of fields
        ],
        bytes
    );

    let mut decoder = VecDecoder::<NoCustomTypeId>::new(&bytes);
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
    let mut encoder = Encoder::<NoCustomTypeId>::new(&mut bytes);
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);

    #[rustfmt::skip]
    assert_eq!(
        vec![
            17, // enum type
            1,  // string size
            65, // "A"
            1,  // number of fields
            9, 2, 0, 0, 0, // field value

            17, // enum type
            1,  // string size
            66, // "B"
            1,  // number of fields
            9, 4, 0, 0, 0, // field value
            
            17, // enum type
            1,  // string size
            67, // "C"
            0,  // number of fields
        ],
        bytes
    );

    let mut decoder = VecDecoder::<NoCustomTypeId>::new(&bytes);
    let a = decoder.decode::<TestEnum>().unwrap();
    let b = decoder.decode::<TestEnum>().unwrap();
    let c = decoder.decode::<TestEnum>().unwrap();

    assert_eq!(TestEnum::A { x: 0, y: 2 }, a);
    assert_eq!(TestEnum::B(0, 4), b);
    assert_eq!(TestEnum::C, c);
}
