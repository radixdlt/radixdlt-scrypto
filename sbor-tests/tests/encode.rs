use sbor::Encode;
use sbor::Encoder;

#[derive(Encode)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Encode)]
pub struct TestStructUnnamed(u32);

#[derive(Encode)]
pub struct TestStructUnit;

#[derive(Encode)]
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

    let mut encoder = Encoder::new();
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);
    let bytes: Vec<u8> = encoder.into();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            17, // struct type
            12, 15, 0, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 78, 97, 109, 101, 100, // struct name
            19, // fields type
            1, 0, // number of fields
            12, 5, 0, 115, 116, 97, 116, 101, // field name
            9, 3, 0, 0, 0, // field value
            
            17,  // struct type
            12, 17, 0, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 85, 110, 110, 97, 109, 101, 100, // struct name
            20,  // fields type
            1, 0,  // number of fields
            9, 3, 0, 0, 0,  // field value
            
            17, // struct type
            12, 14, 0, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 85, 110, 105, 116, // struct name
            21 // fields type
        ],
        bytes
    );
}

#[test]
fn test_encode_enum() {
    let a = TestEnum::A { x: 2, y: 3 };
    let b = TestEnum::B(1);
    let c = TestEnum::C;

    let mut encoder = Encoder::new();
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);
    let bytes: Vec<u8> = encoder.into();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            18, // enum type
            12, 8, 0, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            0, // enum index
            12, 1, 0, 65, // variant name
            19, // fields type
            2, 0,  // number of fields
            12, 1, 0, 120, // field name
            9, 2, 0, 0, 0, // field value
            12, 1, 0, 121,  // field name
            9, 3, 0, 0, 0,  // field value

            18, // enum type
            12, 8, 0, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            1,  // enum index
            12, 1, 0, 66, // variant name
            20, // fields type
            1, 0, // number of fields
            9, 1, 0, 0, 0, // field value
            
            18, // enum type
            12, 8, 0, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            2,  // enum index
            12, 1, 0, 67, // variant name
            21  // fields type
        ],
        bytes
    );
}
