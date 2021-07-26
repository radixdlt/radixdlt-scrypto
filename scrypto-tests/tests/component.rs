use scrypto::kernel::radix_alloc;
use scrypto_derive::component;

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

#[no_mangle]
pub extern "C" fn radix_kernel(_op: u32, _input_ptr: *const u8, _input_len: usize) -> *mut u8 {
    radix_alloc(0)
}
