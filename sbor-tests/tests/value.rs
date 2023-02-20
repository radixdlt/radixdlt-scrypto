#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use serde::Serialize;
use serde_json::{json, to_string, to_value, Value};

#[derive(Sbor)]
pub struct Sample {
    pub a: (),
    pub b: u32,
    pub c: (u8, Vec<u8>),
    pub d: String,
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
fn test_encode_as_json() {
    let sample = Sample {
        a: (),
        b: 1,
        c: (2, vec![3, 4]),
        d: "5".to_string(),
    };
    let bytes = basic_encode(&sample).unwrap();
    let any = basic_decode::<BasicValue>(&bytes).unwrap();

    assert_json_eq(
        any,
        json!({
            "fields": [
                {
                    "fields": [],
                    "type": "Tuple"
                },
                {
                    "type": "U32",
                    "value": 1
                },
                {
                    "fields": [
                        {
                            "type": "U8",
                            "value": 2
                        },
                        {
                            "element_value_kind": {
                                "type": "U8"
                            },
                            "elements": [
                                {
                                    "type": "U8",
                                    "value": 3
                                },
                                {
                                    "type": "U8",
                                    "value": 4
                                }
                            ],
                            "type": "Array"
                        }
                    ],
                    "type": "Tuple"
                },
                {
                    "type": "String",
                    "value": "5"
                }
            ],
            "type": "Tuple"
        }),
    );
}
