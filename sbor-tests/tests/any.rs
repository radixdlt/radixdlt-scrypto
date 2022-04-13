#![cfg_attr(not(feature = "std"), no_std)]

#[rustfmt::skip]
pub mod utils;

use crate::utils::assert_json_eq;
use sbor::rust::vec;
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
            "Struct": [
                "Unit",
                {
                    "U32": 1
                },
                {
                    "Tuple": [
                        {
                            "U8": 2
                        },
                        {
                            "Vec": [
                                7,
                                [
                                    {
                                        "U8": 3
                                    },
                                    {
                                        "U8": 4
                                    }
                                ]
                            ]
                        }
                    ]
                },
                {
                    "String": "5"
                }
            ]
        }),
    );
}
