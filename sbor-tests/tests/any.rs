#![cfg_attr(not(feature = "std"), no_std)]

#[rustfmt::skip]
pub mod utils;

use crate::utils::assert_json_eq;
use sbor::rust::vec;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use serde_json::json;

#[derive(TypeId, Encode, Decode, Describe)]
pub struct Sample {
    pub a: (),
    pub b: u32,
    pub c: (u8, Vec<u8>),
    pub d: String,
}

#[test]
fn test_encode_as_json() {
    let sample = Sample {
        a: (),
        b: 1,
        c: (2, vec![3, 4]),
        d: "5".to_string(),
    };
    let bytes = sbor::encode_with_type(&sample);
    let any = sbor::decode_any(&bytes).unwrap();

    assert_json_eq(
        any,
        json!({
            "fields": [
                {
                    "type": "Unit"
                },
                {
                    "type": "U32",
                    "value": 1
                },
                {
                    "elements": [
                        {
                            "type": "U8",
                            "value": 2
                        },
                        {
                            "elementTypeId": 7,
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
                            "type": "Vec"
                        }
                    ],
                    "type": "Tuple"
                },
                {
                    "type": "String",
                    "value": "5"
                }
            ],
            "type": "Struct"
        }),
    );
}
