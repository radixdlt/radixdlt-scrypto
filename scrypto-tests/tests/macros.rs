extern crate alloc;
use alloc::vec::Vec;

use scrypto::buffer::{bincode_decode, bincode_encode};
use scrypto::constructs::{Blueprint, Component};
use scrypto::kernel::*;
use scrypto::types::Address;
use scrypto::*;

const TEST_LOG_MESSAGE: &'static str = "Hello, Radix!";
const TEST_BLUEPRINT_ADDRESS: &'static str =
    "050377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee94";
const TEST_COMPONENT_ADDRESS: &'static str =
    "06c46576324df8c76f6d83611974e8d26a12fe648280c19974c979";
const TEST_COMPONENT_NAME: &'static str = "ComponentName";
const TEST_COMPONENT_METHOD: &'static str = "method";
const TEST_RETURN: &'static str = "5";

#[no_mangle]
pub extern "C" fn radix_kernel(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8 {
    let mut input_bytes = Vec::<u8>::with_capacity(input_len);
    unsafe {
        core::ptr::copy(input_ptr, input_bytes.as_mut_ptr(), input_len);
        input_bytes.set_len(input_len);
    }
    let output_bytes;

    match op {
        EMIT_LOG => {
            let input: EmitLogInput = bincode_decode(&input_bytes);
            assert_eq!(input.message, TEST_LOG_MESSAGE);

            let output = EmitLogOutput {};
            output_bytes = bincode_encode(&output);
        }
        CALL_BLUEPRINT => {
            let input: CallBlueprintInput = bincode_decode(&input_bytes);
            assert_eq!(input.blueprint, Address::from(TEST_BLUEPRINT_ADDRESS));
            assert_eq!(input.component, TEST_COMPONENT_NAME);
            assert_eq!(input.method, TEST_COMPONENT_METHOD);

            let output = CallBlueprintOutput {
                rtn: Vec::from(TEST_RETURN.as_bytes()),
            };
            output_bytes = bincode_encode(&output);
        }
        GET_COMPONENT_INFO => {
            let input: GetComponentInfoInput = bincode_decode(&input_bytes);
            assert_eq!(input.component, Address::from(TEST_COMPONENT_ADDRESS));

            let output = GetComponentInfoOutput {
                result: Some(ComponentInfo {
                    blueprint: Address::from(TEST_BLUEPRINT_ADDRESS),
                    kind: TEST_COMPONENT_NAME.to_string(),
                }),
            };
            output_bytes = bincode_encode(&output);
        }
        _ => panic!("Unexpected operation: {}", op),
    }

    let output_ptr = radix_alloc(output_bytes.len());
    unsafe {
        core::ptr::copy(output_bytes.as_ptr(), output_ptr, output_bytes.len());
    }
    output_ptr
}

#[test]
fn test_logging() {
    error!("Hello, {}!", "Radix");
    warn!("Hello, {}!", "Radix");
    info!("Hello, {}!", "Radix");
    debug!("Hello, {}!", "Radix");
    trace!("Hello, {}!", "Radix");
}

#[test]
fn test_call_blueprint() {
    let blueprint = Blueprint::from(Address::from(TEST_BLUEPRINT_ADDRESS));
    let rtn = call_blueprint!(
        i32,
        blueprint,
        TEST_COMPONENT_NAME,
        TEST_COMPONENT_METHOD,
        123
    );
    assert_eq!(rtn, TEST_RETURN.parse::<i32>().unwrap());
}

#[test]
fn test_call_component() {
    let component = Component::from(Address::from(TEST_COMPONENT_ADDRESS));
    let rtn = call_component!(i32, component, TEST_COMPONENT_METHOD, 456);
    assert_eq!(rtn, TEST_RETURN.parse::<i32>().unwrap());
}
