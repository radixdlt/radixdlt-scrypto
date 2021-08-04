#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec;

use scrypto::buffer::*;
use scrypto::component;
use serde::Serialize;
use serde_json::{json, to_value, Value};

component! {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> Self {
            Self {
                state: 0
            }
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
    let bytes = scrypto_copy(ptr);
    scrypto_free(ptr);
    let abi: scrypto::abi::Component = scrypto_decode(&bytes);

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
                "type": "Struct",
                "name": "Simple",
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
pub extern "C" fn kernel_main(_op: u32, _input_ptr: *const u8, _input_len: usize) -> *mut u8 {
    scrypto_alloc(0)
}
