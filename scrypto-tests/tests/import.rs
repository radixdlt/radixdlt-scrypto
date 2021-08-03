#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use scrypto::buffer::radix_alloc;
use scrypto::import;

// base directory: `scrypto-derive`
import! { "../scrypto-tests/tests/abi.json" }

#[test]
#[should_panic] // asserts it compiles
fn test_import_from_abi() {
    let instance = Sample::from_address("".into());

    let arg1 = Floor { x: 5, y: 12 };
    let arg2 = (1u8, 2u16);
    let arg3 = Vec::<String>::new();
    let arg4 = 5;
    let arg5 = Hello::A { x: 1 };

    let _rtn = instance.calculate_volume(arg1, arg2, arg3, arg4, arg5);
}

#[no_mangle]
pub extern "C" fn kernel_main(_op: u32, _input_ptr: *const u8, _input_len: usize) -> *mut u8 {
    radix_alloc(0)
}
