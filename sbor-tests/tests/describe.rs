#![cfg_attr(not(feature = "std"), no_std)]

#[rustfmt::skip]
pub mod utils;

use crate::utils::assert_json_eq;
use sbor::rust::vec;
use sbor::Describe;
use serde_json::json;

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
