#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;
use serde::Serialize;
use serde_json::{json, to_string, to_value, Value};

#[derive(LegacyDescribe)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(LegacyDescribe)]
pub struct TestStructUnnamed(u32);

#[derive(LegacyDescribe)]
pub struct TestStructUnit;

#[derive(LegacyDescribe)]
pub enum TestEnum {
    A,
    B(u32),
    C { x: u32, y: u32 },
}

pub fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    let actual = to_value(&actual).unwrap();
    if actual != expected {
        panic!(
            "Mismatching JSONs:\nActual:\n{}\nExpected:\n{}\n",
            to_string(&actual).unwrap(),
            to_string(&expected).unwrap()
        );
    }
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
                "named": [
                    [
                        "state",
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
fn test_describe_enum() {
    assert_json_eq(
        TestEnum::describe(),
        json!({
            "type": "Enum",
            "name": "TestEnum",
            "variants": [
                {
                    "name": "A",
                    "fields": {
                        "type": "Unit"
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
                        "type": "Named",
                        "named": [
                            [
                                "x",
                                {
                                    "type": "U32"
                                }
                            ],
                            [
                                "y",
                                {
                                    "type": "U32"
                                }
                            ]
                        ]
                    }
                }
            ]
        }),
    );
}
