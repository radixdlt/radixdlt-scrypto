use sbor::describe::Fields;
use sbor::Type;
use scrypto::abi::{Function, Method};
use scrypto::prelude::*;

static mut LARGE: [u8; 4] = (u32::MAX / 2).to_le_bytes();
static mut MAX: [u8; 4] = u32::MAX.to_le_bytes();
static mut ZERO: [u8; 4] = [0, 0, 0, 0];

#[no_mangle]
pub extern "C" fn LargeReturnSize_main(input: *const u8) -> *mut u8 {
    unsafe { LARGE.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_main(input: *const u8) -> *mut u8 {
    unsafe { MAX.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_main(input: *const u8) -> *mut u8 {
    unsafe { ZERO.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn LargeReturnSize_abi(input: *const u8) -> *mut u8 {
    let blueprint_type = Type::Struct {
        name: "LargeReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi: (Type, Vec<Function>, Vec<Method>) = (blueprint_type, vec![], vec![]);

    // serialize the output
    let output_bytes = ::scrypto::buffer::scrypto_encode_with_size_prefix(&abi);

    // return the output wrapped in a radix-style buffer
    ::scrypto::buffer::scrypto_wrap(output_bytes)
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_abi(input: *const u8) -> *mut u8 {
    let blueprint_type = Type::Struct {
        name: "MaxReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi: (Type, Vec<Function>, Vec<Method>) = (blueprint_type, vec![], vec![]);

    // serialize the output
    let output_bytes = ::scrypto::buffer::scrypto_encode_with_size_prefix(&abi);

    // return the output wrapped in a radix-style buffer
    ::scrypto::buffer::scrypto_wrap(output_bytes)
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_abi(input: *const u8) -> *mut u8 {
    let blueprint_type = Type::Struct {
        name: "ZeroReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi: (Type, Vec<Function>, Vec<Method>) = (blueprint_type, vec![], vec![]);

    // serialize the output
    let output_bytes = ::scrypto::buffer::scrypto_encode_with_size_prefix(&abi);

    // return the output wrapped in a radix-style buffer
    ::scrypto::buffer::scrypto_wrap(output_bytes)
}
