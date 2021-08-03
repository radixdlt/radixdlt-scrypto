#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;

use scrypto::buffer::{scrypto_alloc, scrypto_decode, scrypto_encode};
use scrypto::constructs::{Blueprint, Component};
use scrypto::kernel::*;
use scrypto::types::Address;
use scrypto::*;

const LOG_MESSAGE: &'static str = "Hello, Radix!";
const BLUEPRINT_ADDRESS: &'static str = "050377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee94";
const COMPONENT_ADDRESS: &'static str = "06c46576324df8c76f6d83611974e8d26a12fe648280c19974c979";
const COMPONENT_NAME: &'static str = "ComponentName";
const COMPONENT_METHOD: &'static str = "method";
const RETURN: i32 = 5;

#[no_mangle]
pub extern "C" fn kernel_main(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8 {
    let mut input_bytes = Vec::<u8>::with_capacity(input_len);
    unsafe {
        core::ptr::copy(input_ptr, input_bytes.as_mut_ptr(), input_len);
        input_bytes.set_len(input_len);
    }
    let output_bytes;

    match op {
        EMIT_LOG => {
            let input: EmitLogInput = scrypto_decode(&input_bytes);
            assert_eq!(input.message, LOG_MESSAGE);

            let output = EmitLogOutput {};
            output_bytes = scrypto_encode(&output);
        }
        CALL_BLUEPRINT => {
            let input: CallBlueprintInput = scrypto_decode(&input_bytes);
            assert_eq!(input.blueprint, Address::from(BLUEPRINT_ADDRESS));
            assert_eq!(input.component, COMPONENT_NAME);
            assert_eq!(input.method, COMPONENT_METHOD);

            let output = CallBlueprintOutput {
                rtn: scrypto_encode(&RETURN),
            };
            output_bytes = scrypto_encode(&output);
        }
        GET_COMPONENT_INFO => {
            let input: GetComponentInfoInput = scrypto_decode(&input_bytes);
            assert_eq!(input.component, Address::from(COMPONENT_ADDRESS));

            let output = GetComponentInfoOutput {
                result: Some(ComponentInfo {
                    blueprint: Address::from(BLUEPRINT_ADDRESS),
                    kind: COMPONENT_NAME.to_string(),
                }),
            };
            output_bytes = scrypto_encode(&output);
        }
        _ => panic!("Unexpected operation: {}", op),
    }

    let output_ptr = scrypto_alloc(output_bytes.len());
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
    let blueprint = Blueprint::from(Address::from(BLUEPRINT_ADDRESS));
    let rtn = call_blueprint!(i32, blueprint, COMPONENT_NAME, COMPONENT_METHOD, 123);
    assert_eq!(rtn, RETURN);
}

#[test]
fn test_call_component() {
    let component = Component::from(Address::from(COMPONENT_ADDRESS));
    let rtn = call_component!(i32, component, COMPONENT_METHOD, 456);
    assert_eq!(rtn, RETURN);
}
