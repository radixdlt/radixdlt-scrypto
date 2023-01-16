use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::wasm::*;
use sbor::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: ScryptoDecode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
        let output_ptr = radix_engine(input_ptr);
        scrypto_decode_from_buffer::<V>(output_ptr).unwrap()
    }
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine_to_raw(input: RadixEngineInput) -> Vec<u8> {
    use crate::buffer::{scrypto_buffer_to_vec, *};

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
        let output_ptr = radix_engine(input_ptr);
        scrypto_buffer_to_vec(output_ptr)
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: ScryptoDecode>(_input: RadixEngineInput) -> V {
    todo!()
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine_to_raw(_input: RadixEngineInput) -> Vec<u8> {
    todo!()
}
