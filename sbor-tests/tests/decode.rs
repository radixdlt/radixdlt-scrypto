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
    ];

    let mut decoder = Decoder::new(&bytes);
    let a = TestEnum::decode(&mut decoder).unwrap();
    let b = TestEnum::decode(&mut decoder).unwrap();
    let c = TestEnum::decode(&mut decoder).unwrap();

    assert_eq!(TestEnum::A { x: 2, y: 3 }, a);
    assert_eq!(TestEnum::B(1), b);
    assert_eq!(TestEnum::C, c);
}
