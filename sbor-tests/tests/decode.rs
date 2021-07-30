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
    A,
    B(u32),
    C { x: u32, y: u32 },
}

#[test]
fn test_decode_struct() {
    #[rustfmt::skip]
    let bytes = vec![
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
    ];

    let mut decoder = Decoder::new(&bytes);
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
    ];

    let mut decoder = Decoder::new(&bytes);
    let a = TestEnum::decode(&mut decoder).unwrap();
    let b = TestEnum::decode(&mut decoder).unwrap();
    let c = TestEnum::decode(&mut decoder).unwrap();

    assert_eq!(TestEnum::A, a);
    assert_eq!(TestEnum::B(1), b);
    assert_eq!(TestEnum::C { x: 2, y: 3 }, c);
}
