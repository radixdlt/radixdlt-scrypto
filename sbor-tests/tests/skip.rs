#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use serde::Serialize;
use serde_json::{json, to_value, Value};

#[derive(Debug, PartialEq, Encode, Decode, Describe)]
pub struct TestStructNamed {
    #[allow(unused_variables)]
    #[sbor(skip)]
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode, Describe)]
pub struct TestStructUnnamed(#[sbor(skip)] u32, u32);

#[derive(Debug, PartialEq, Encode, Decode, Describe)]
pub struct TestStructUnit;

#[derive(Debug, PartialEq, Encode, Decode, Describe)]
pub enum TestEnum {
    A {
        #[sbor(skip)]
        x: u32,
        y: u32,
    },
    B(#[sbor(skip)] u32, u32),
    C,
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_struct() {
    let a = TestStructNamed { x: 1, y: 2 };
    let b = TestStructUnnamed(3, 4);
    let c = TestStructUnit;

    let mut encoder = Encoder::with_metadata(Vec::with_capacity(512));
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);
    let bytes: Vec<u8> = encoder.into();

    #[rustfmt::skip]
    assert_eq!(
        vec![
          20, // struct type
          22, // fields type
          1, 0, 0, 0, // number of fields
          1, 0, 0, 0, 121, // field name
          9, 2, 0, 0, 0, // field value
          
          20,  // struct type
          23,  // fields type
          1, 0, 0, 0,  // number of fields
          9, 4, 0, 0, 0,  // field value
          
          20, // struct type
          24 // fields type
        ],
        bytes
    );

    let mut decoder = Decoder::with_metadata(&bytes);
    let a = TestStructNamed::decode(&mut decoder).unwrap();
    let b = TestStructUnnamed::decode(&mut decoder).unwrap();
    let c = TestStructUnit::decode(&mut decoder).unwrap();

    assert_eq!(TestStructNamed { x: 0, y: 2 }, a);
    assert_eq!(TestStructUnnamed(0, 4), b);
    assert_eq!(TestStructUnit {}, c);

    assert_json_eq(
        TestStructNamed::describe(),
        json!({
          "type": "Struct",
          "name": "TestStructNamed",
          "fields": {
            "type": "Named",
            "named": [
              [
                "y",
                {
                  "type": "U32"
                }
              ]
            ]
          }
        }),
    );
    assert_json_eq(
        TestStructUnnamed::describe(),
        json!({
          "type": "Struct",
          "name": "TestStructUnnamed",
          "fields": {
            "type": "Unnamed",
            "unnamed": [
              {
                "type": "U32"
              }
            ]
          }
        }),
    );
    assert_json_eq(
        TestStructUnit::describe(),
        json!({
          "type": "Struct",
          "name": "TestStructUnit",
          "fields": {
            "type": "Unit"
          }
        }),
    );
}

#[test]
fn test_enum() {
    let a = TestEnum::A { x: 1, y: 2 };
    let b = TestEnum::B(3, 4);
    let c = TestEnum::C;

    let mut encoder = Encoder::with_metadata(Vec::with_capacity(512));
    a.encode(&mut encoder);
    b.encode(&mut encoder);
    c.encode(&mut encoder);
    let bytes: Vec<u8> = encoder.into();

    #[rustfmt::skip]
    assert_eq!(
        vec![
            21, // enum type
            0, // enum index
            1, 0, 0, 0, 65, // variant name
            22, // fields type
            1, 0, 0, 0,  // number of fields
            1, 0, 0, 0, 121, // field name
            9, 2, 0, 0, 0, // field value

            21, // enum type
            1,  // enum index
            1, 0, 0, 0, 66, // variant name
            23, // fields type
            1, 0, 0, 0, // number of fields
            9, 4, 0, 0, 0, // field value
            
            21, // enum type
            2,  // enum index
            1, 0, 0, 0, 67, // variant name
            24  // fields type
        ],
        bytes
    );

    let mut decoder = Decoder::with_metadata(&bytes);
    let a = TestEnum::decode(&mut decoder).unwrap();
    let b = TestEnum::decode(&mut decoder).unwrap();
    let c = TestEnum::decode(&mut decoder).unwrap();

    assert_eq!(TestEnum::A { x: 0, y: 2 }, a);
    assert_eq!(TestEnum::B(0, 4), b);
    assert_eq!(TestEnum::C, c);

    assert_json_eq(
        TestEnum::describe(),
        json!({
          "type": "Enum",
          "name": "TestEnum",
          "variants": [
            {
              "name": "A",
              "fields": {
                "type": "Named",
                "named": [
                  [
                    "y",
                    {
                      "type": "U32"
                    }
                  ]
                ]
              }
            },
            {
              "name": "B",
              "fields": {
                "type": "Unnamed",
                "unnamed": [
                  {
                    "type": "U32"
                  }
                ]
              }
            },
            {
              "name": "C",
              "fields": {
                "type": "Unit"
              }
            }
          ]
        }),
    );
}
