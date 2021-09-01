#![cfg_attr(not(feature = "std"), no_std)]

use sbor::parse_any;
use sbor::rust::vec;
use serde::Serialize;
use serde_json::{json, to_value, Value};

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_parse_to_json() {
    assert_json_eq(
        parse_any(&vec![
            20, 22, 23, 0, 0, 0, 0, 1, 1, 2, 1, 3, 2, 0, 4, 3, 0, 0, 0, 5, 4, 0, 0, 0, 0, 0, 0, 0,
            6, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 6, 8, 7, 0, 9, 8, 0, 0, 0, 10, 9,
            0, 0, 0, 0, 0, 0, 0, 11, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 3, 0, 0,
            0, 97, 98, 99, 16, 1, 9, 1, 0, 0, 0, 17, 9, 1, 0, 0, 0, 18, 9, 3, 0, 0, 0, 1, 0, 0, 0,
            2, 0, 0, 0, 3, 0, 0, 0, 19, 2, 0, 0, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, 20, 22, 1, 0, 0,
            0, 9, 1, 0, 0, 0, 21, 0, 22, 1, 0, 0, 0, 9, 1, 0, 0, 0, 21, 1, 23, 1, 0, 0, 0, 9, 2, 0,
            0, 0, 21, 2, 24, 32, 9, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 36, 1, 0, 0, 0, 9, 9, 1, 0,
            0, 0, 2, 0, 0, 0,
        ])
        .unwrap(),
        json!({
          "type": "Struct",
          "fields": {
            "type": "Named",
            "named": [
              {
                "type": "Unit"
              },
              {
                "type": "Bool",
                "value": true
              },
              {
                "type": "I8",
                "value": 1
              },
              {
                "type": "I16",
                "value": 2
              },
              {
                "type": "I32",
                "value": 3
              },
              {
                "type": "I64",
                "value": 4
              },
              {
                "type": "I128",
                "value": "5"
              },
              {
                "type": "U8",
                "value": 6
              },
              {
                "type": "U16",
                "value": 7
              },
              {
                "type": "U32",
                "value": 8
              },
              {
                "type": "U64",
                "value": 9
              },
              {
                "type": "U128",
                "value": "10"
              },
              {
                "type": "String",
                "value": "abc"
              },
              {
                "type": "Option",
                "value": {
                  "type": "U32",
                  "value": 1
                }
              },
              {
                "type": "Box",
                "value": {
                  "type": "U32",
                  "value": 1
                }
              },
              {
                "type": "Array",
                "elements": [
                  {
                    "type": "U32",
                    "value": 1
                  },
                  {
                    "type": "U32",
                    "value": 2
                  },
                  {
                    "type": "U32",
                    "value": 3
                  }
                ]
              },
              {
                "type": "Tuple",
                "elements": [
                  {
                    "type": "U32",
                    "value": 1
                  },
                  {
                    "type": "U32",
                    "value": 2
                  }
                ]
              },
              {
                "type": "Struct",
                "fields": {
                  "type": "Named",
                  "named": [
                    {
                      "type": "U32",
                      "value": 1
                    }
                  ]
                }
              },
              {
                "type": "Enum",
                "index": 0,
                "fields": {
                  "type": "Named",
                  "named": [
                    {
                      "type": "U32",
                      "value": 1
                    }
                  ]
                }
              },
              {
                "type": "Enum",
                "index": 1,
                "fields": {
                  "type": "Unnamed",
                  "unnamed": [
                    {
                      "type": "U32",
                      "value": 2
                    }
                  ]
                }
              },
              {
                "type": "Enum",
                "index": 2,
                "fields": {
                  "type": "Unit"
                }
              },
              {
                "type": "Vec",
                "elements": [
                  {
                    "type": "U32",
                    "value": 1
                  },
                  {
                    "type": "U32",
                    "value": 2
                  }
                ]
              },
              {
                "type": "HashMap",
                "elements": [
                  [
                    {
                      "type": "U32",
                      "value": 1
                    },
                    {
                      "type": "U32",
                      "value": 2
                    }
                  ]
                ]
              }
            ]
          }
        }),
    );
}
