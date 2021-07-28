use scrypto::component;
use scrypto::kernel::*;

mod utils;
use serde_json::json;
use utils::json_eq;

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
    let abi = radix_copy(ptr);
    radix_free(ptr);

    json_eq(
        json!({
          "name": "Simple",
          "methods": [
            {
              "name": "new",
              "kind": "Functional",
              "mutability": "Immutable",
              "inputs": [],
              "output": {
                "type": "Struct",
                "name": "Simple",
                "fields": {
                  "type": "Named",
                  "fields": {
                    "state": {
                      "type": "U32"
                    }
                  }
                }
              }
            },
            {
              "name": "get_state",
              "kind": "Stateful",
              "mutability": "Immutable",
              "inputs": [],
              "output": {
                "type": "U32"
              }
            },
            {
              "name": "set_state",
              "kind": "Stateful",
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
        serde_json::from_slice::<scrypto::abi::Component>(&abi).unwrap(),
    );
}

#[no_mangle]
pub extern "C" fn radix_kernel(_op: u32, _input_ptr: *const u8, _input_len: usize) -> *mut u8 {
    radix_alloc(0)
}
