use scrypto::prelude::*;
use sbor::Type;
use sbor::describe::Fields;

static mut LARGE: [u8; 4] = (u32::MAX / 2).to_le_bytes();
static mut MAX: [u8; 4] = u32::MAX.to_le_bytes();
static mut ZERO: [u8; 4] = [0, 0, 0, 0];

#[no_mangle]
pub extern "C" fn LargeReturnSize_main() -> *mut u8 {
    unsafe { LARGE.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_main() -> *mut u8 {
    unsafe { MAX.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_main() -> *mut u8 {
    unsafe { ZERO.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn package_init() -> *mut u8 {
    let mut blueprints: HashMap<String, Type> = HashMap::new();
    blueprints.insert("LargeReturnSize".to_string(), Type::Struct { name: "LargeReturnSize".to_string(), fields: Fields::Unit });
    blueprints.insert("MaxReturnSize".to_string(), Type::Struct { name: "MaxReturnSize".to_string(), fields: Fields::Unit });
    blueprints.insert("ZeroReturnSize".to_string(), Type::Struct { name: "ZeroReturnSize".to_string(), fields: Fields::Unit });

    // serialize the output
    let output_bytes = ::scrypto::buffer::scrypto_encode_for_radix_engine(&blueprints);

    // return the output wrapped in a radix-style buffer
    ::scrypto::buffer::scrypto_wrap(output_bytes)
}