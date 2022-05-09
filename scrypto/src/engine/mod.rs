/// Radix Engine System APIs.
pub mod api;
/// Types and functions shared by both Scrypto and Radix Engine.
pub mod types;

use sbor::Decode;

use crate::engine::api::RadixEngineInput;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: Decode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};
    use crate::engine::api::radix_engine;

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input);
        let output_ptr = radix_engine(input_ptr);
        scrypto_decode_from_buffer::<V>(output_ptr).unwrap()
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: Decode>(_invocation: RadixEngineInput) -> V {
    todo!()
}
