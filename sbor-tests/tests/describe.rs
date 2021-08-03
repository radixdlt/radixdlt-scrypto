#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec;

use sbor::Describe;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Describe)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Describe)]
pub struct TestStructUnnamed(u32);

#[derive(Describe)]
pub struct TestStructUnit;

#[derive(Describe)]
pub enum TestEnum {
    A,
    B(u32),
    C { x: u32, y: u32 },
}

pub fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    let actual_json = serde_json::to_value(&actual).unwrap();
    assert_eq!(actual_json, expected);
}

#[test]
fn test_describe_struct() {
    assert_json_eq(
        TestStructNamed::describe(),
        json!({
          "type": "Struct",
          "name": "TestStructNamed",
          "fields": {
            "type": "Named",
            "fields": {
              "state": {
                "type": "U32"
              }
            }
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
            "fields": [
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
fn test_describe_enum() {
    assert_json_eq(
        TestEnum::describe(),
        json!({
          "type": "Enum",
          "name": "TestEnum",
          "variants": {
            "A": {
              "type": "Unit"
            },
            "B": {
              "type": "Unnamed",
              "fields": [
                {
                  "type": "U32"
                }
              ]
            },
            "C": {
              "type": "Named",
              "fields": {
                "x": {
                  "type": "U32"
                },
                "y": {
                  "type": "U32"
                }
              }
            }
          }
        }),
    );
}
