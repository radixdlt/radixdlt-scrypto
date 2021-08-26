#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::buffer::{scrypto_decode, scrypto_encode, scrypto_wrap};
use scrypto::constructs::{Blueprint, Component};
use scrypto::kernel::*;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::*;

const LOG_MESSAGE: &'static str = "Hello, Radix!";
const PACKAGE_ADDRESS: &'static str = "050377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee94";
const COMPONENT_ADDRESS: &'static str = "06c46576324df8c76f6d83611974e8d26a12fe648280c19974c979";
const BLUEPRINT_NAME: &'static str = "BlueprintName";
const FUNCTION_NAME: &'static str = "function";
const METHOD_NAME: &'static str = "method";
const RETURN: i32 = 5;

#[no_mangle]
pub extern "C" fn kernel(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8 {
    let mut input_bytes = Vec::<u8>::with_capacity(input_len);
    unsafe {
        core::ptr::copy(input_ptr, input_bytes.as_mut_ptr(), input_len);
        input_bytes.set_len(input_len);
    }
    let output_bytes;

    match op {
        EMIT_LOG => {
            let input: EmitLogInput = scrypto_decode(&input_bytes).unwrap();
            assert_eq!(input.message, LOG_MESSAGE);

            let output = EmitLogOutput {};
            output_bytes = scrypto_encode(&output);
        }
        CALL_FUNCTION => {
            let input: CallFunctionInput = scrypto_decode(&input_bytes).unwrap();
            assert_eq!(input.package, Address::from_str(PACKAGE_ADDRESS).unwrap());
            assert_eq!(input.blueprint, BLUEPRINT_NAME);
            assert_eq!(input.function, FUNCTION_NAME);

            let output = CallFunctionOutput {
                rtn: scrypto_encode(&RETURN),
            };
            output_bytes = scrypto_encode(&output);
        }
        CALL_METHOD => {
            let input: CallMethodInput = scrypto_decode(&input_bytes).unwrap();
            assert_eq!(
                input.component,
                Address::from_str(COMPONENT_ADDRESS).unwrap()
            );
            assert_eq!(input.method, METHOD_NAME);

            let output = CallMethodOutput {
                rtn: scrypto_encode(&RETURN),
            };
            output_bytes = scrypto_encode(&output);
        }
        GET_COMPONENT_INFO => {
            let input: GetComponentInfoInput = scrypto_decode(&input_bytes).unwrap();
            assert_eq!(
                input.component,
                Address::from_str(COMPONENT_ADDRESS).unwrap()
            );

            let output = GetComponentInfoOutput {
                package: Address::from_str(PACKAGE_ADDRESS).unwrap(),
                blueprint: BLUEPRINT_NAME.to_string(),
            };
            output_bytes = scrypto_encode(&output);
        }
        _ => panic!("Unexpected operation: {}", op),
    }

    scrypto_wrap(&output_bytes)
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
fn test_call_function() {
    let blueprint = Blueprint::from(Address::from_str(PACKAGE_ADDRESS).unwrap(), BLUEPRINT_NAME);
    let rtn: i32 = blueprint.call(FUNCTION_NAME, args!(123));
    assert_eq!(rtn, RETURN);
}

#[test]
fn test_call_method() {
    let component = Component::from(Address::from_str(COMPONENT_ADDRESS).unwrap());
    component.call::<i32>(METHOD_NAME, args!(456));
}
