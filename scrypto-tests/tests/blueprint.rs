#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::abi::*;
use scrypto::buffer::*;
use scrypto::prelude::*;
use serde::Serialize;
use serde_json::{json, to_string, to_value, Value};

blueprint! {
  struct Empty {
  }

  impl Empty {

  }
}

blueprint! {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> ComponentAddress {
            Self {
                state: 0
            }
            .instantiate()
            .globalize()
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }

        pub fn custom_types() -> (Decimal, PackageAddress, KeyValueStore<String, String>, Hash, Bucket, Proof, Vault) {
            todo!()
        }
    }
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
fn test_simple_abi() {
    let ptr = Simple_abi(core::ptr::null_mut::<u8>());
    let abi: BlueprintAbi = scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap());

    assert_json_eq(
        abi,
        json!({
          "fns": [
            {
              "export_name": "Simple_new",
              "ident": "new",
              "input": {
                "fields": {
                  "named": [],
                  "type": "Named"
                },
                "name": "Simple_new_Input",
                "type": "Struct"
              },
              "mutability": null,
              "output": {
                "type": "ComponentAddress"
              }
            },
            {
              "export_name": "Simple_get_state",
              "ident": "get_state",
              "input": {
                "fields": {
                  "named": [],
                  "type": "Named"
                },
                "name": "Simple_get_state_Input",
                "type": "Struct"
              },
              "mutability": "Immutable",
              "output": {
                "type": "U32"
              }
            },
            {
              "export_name": "Simple_set_state",
              "ident": "set_state",
              "input": {
                "fields": {
                  "named": [
                    [
                      "arg0",
                      {
                        "type": "U32"
                      }
                    ]
                  ],
                  "type": "Named"
                },
                "name": "Simple_set_state_Input",
                "type": "Struct"
              },
              "mutability": "Mutable",
              "output": {
                "fields": {
                  "type": "Unit"
                },
                "type": "Tuple"
              }
            },
            {
              "export_name": "Simple_custom_types",
              "ident": "custom_types",
              "input": {
                "fields": {
                  "named": [],
                  "type": "Named"
                },
                "name": "Simple_custom_types_Input",
                "type": "Struct"
              },
              "mutability": null,
              "output": {
                "element_types": [
                  {
                    "type": "Decimal"
                  },
                  {
                    "type": "PackageAddress"
                  },
                  {
                    "key_type": {
                      "type": "String"
                    },
                    "type": "KeyValueStore",
                    "value_type": {
                      "type": "String"
                    }
                  },
                  {
                    "type": "Hash"
                  },
                  {
                    "type": "Bucket"
                  },
                  {
                    "type": "Proof"
                  },
                  {
                    "type": "Vault"
                  }
                ],
                "type": "Tuple"
              }
            }
          ],
          "structure": {
            "fields": {
              "named": [
                [
                  "state",
                  {
                    "type": "U32"
                  }
                ]
              ],
              "type": "Named"
            },
            "name": "Simple",
            "type": "Struct"
          }
        }),
    );
}
