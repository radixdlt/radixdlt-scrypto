#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::rust::string::ToString;
use sbor::Encode;
use sbor::Encoder;
use sbor::TypeId;

#[derive(TypeId, Encode)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(TypeId, Encode)]
pub struct TestStructUnnamed(u32);

#[derive(TypeId, Encode)]
pub struct TestStructUnit;

#[derive(TypeId, Encode)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[test]
fn test_encode_struct() {
    let a = TestStructNamed { state: 3 };
    let b = TestStructUnnamed(3);
    let c = TestStructUnit {};

    let mut bytes = Vec::with_capacity(512);
    let mut encoder = Encoder::with_type(&mut bytes);
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);

    #[rustfmt::skip]
    assert_eq!(
        vec![
            16, // struct type 
            1, 0, 0, 0, // number of fields
            9, 3, 0, 0, 0, // field value
            
            16,  // struct type 
            1, 0, 0, 0,  // number of fields
            9, 3, 0, 0, 0,  // field value
            
            16, // struct type
            0, 0, 0, 0,  // number of fields
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
    let mut encoder = Encoder::with_type(&mut bytes);
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);

    #[rustfmt::skip]
    assert_eq!(
        vec![
            17, // enum type
            1, 0, 0, 0, // string size
            65, // "A"
            2, 0, 0, 0,  // number of fields
            9, 2, 0, 0, 0, // field value
            9, 3, 0, 0, 0,  // field value

            17, // enum type
            1, 0, 0, 0,  // string size
            66, // "B"
            1, 0, 0, 0, // number of fields
            9, 1, 0, 0, 0, // field value
            
            17, // enum type
            1, 0, 0, 0,  // string size
            67, // "C"
            0, 0, 0, 0,  // number of fields
        ],
        bytes
    );
}
