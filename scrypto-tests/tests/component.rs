#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec;

use scrypto::buffer::*;
use scrypto::component;
use scrypto::constructs::*;
use scrypto::types::*;
use serde::Serialize;
use serde_json::{json, to_value, Value};

component! {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> Address {
            Component::new("Simple", Self {
                state: 0
            })
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }
    }
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
#[should_panic] // asserts it compiles
fn test_simple_component() {
    let mut stub = SimpleStub::from_address("".into());
    let x = stub.get_state();
    stub.set_state(x + 1);
}

#[test]
fn test_simple_component_abi() {
    let ptr = Simple_abi();
    let abi: scrypto::abi::Component = scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap());

    assert_json_eq(
        abi,
        json!({
          "name": "Simple",
          "methods": [
            {
              "name": "new",
              "mutability": "Stateless",
              "inputs": [],
              "output": {
                "type": "Enum",
                "name": "Address",
                "variants": [
                  {
                    "name": "System",
                    "fields": {
                      "type": "Unit"
                    }
                  },
                  {
                    "name": "RadixToken",
                    "fields": {
                      "type": "Unit"
                    }
                  },
                  {
                    "name": "Resource",
                    "fields": {
                      "type": "Unnamed",
                      "unnamed": [
                        {
                          "type": "Array",
                          "element": {
                            "type": "U8"
                          },
                          "length": 26
                        }
                      ]
                    }
                  },
                  {
                    "name": "Account",
                    "fields": {
                      "type": "Unnamed",
                      "unnamed": [
                        {
                          "type": "Array",
                          "element": {
                            "type": "U8"
                          },
                          "length": 33
                        }
                      ]
                    }
                  },
                  {
                    "name": "Blueprint",
                    "fields": {
                      "type": "Unnamed",
                      "unnamed": [
                        {
                          "type": "Array",
                          "element": {
                            "type": "U8"
                          },
                          "length": 26
                        }
                      ]
                    }
                  },
                  {
                    "name": "Component",
                    "fields": {
                      "type": "Unnamed",
                      "unnamed": [
                        {
                          "type": "Array",
                          "element": {
                            "type": "U8"
                          },
                          "length": 26
                        }
                      ]
                    }
                  }
                ]
              }
            },
            {
              "name": "get_state",
              "mutability": "Immutable",
              "inputs": [],
              "output": {
                "type": "U32"
              }
            },
            {
              "name": "set_state",
              "mutability": "Mutable",
              "inputs": [
                {
                  "type": "U32"
                }
              ],
              "output": {
                "type": "Unit"
              }
            }
          ]
        }),
    );
}

#[no_mangle]
pub extern "C" fn kernel(_op: u32, _input_ptr: *const u8, _input_len: usize) -> *mut u8 {
    scrypto_alloc(0)
}
