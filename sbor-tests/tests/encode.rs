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
    A,
    B(u32),
    C { x: u32, y: u32 },
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
            17, // struct #a
            12, 0, 15, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 78, 97, 109, 101, 100, // struct name
            19,  // fields type
            0, 1, // number of fields
            12, 0, 5, 115, 116, 97, 116, 101, // field name #0
            9, 0, 0, 0, 3,  // field value #0

            17, // struct #b
            12, 0, 17, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 85, 110, 110, 97, 109, 101, 100, // struct name
            20,  // fields type
            0, 1, // number of fields
            9, 0, 0, 0, 3,  // field value #0

            17, // struct #c
            12, 0, 14, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 85, 110, 105, 116, // struct name
            21   // fields type
        ],
        bytes
    );
}

#[test]
fn test_encode_enum() {
    let a = TestEnum::A;
    let b = TestEnum::B(1);
    let c = TestEnum::C { x: 2, y: 3 };

    let mut encoder = Encoder::new();
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);
    let bytes: Vec<u8> = encoder.into();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            18, // enum #a
            12, 0, 8, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            12, 0, 1, 65, // variant name
            21, // variant fields type
            
            18, // enum #b
            12, 0, 8, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            12, 0, 1, 66, // variant name
            20, // variant fields type
            0, 1, // number of fields
            9, 0, 0, 0, 1, // fields

            18, // enum #c
            12, 0, 8, 84, 101, 115, 116, 69, 110, 117, 109, // enum name
            12, 0, 1, 67, // variant name
            19, // variant fields type
            0, 2, // number of fields
            12, 0, 1, 120, // field name #0
            9, 0, 0, 0, 2, // field value #0
            12, 0, 1, 121, // field name #1
            9, 0, 0, 0, 3  // field value #1
        ],
        bytes
    );
}
