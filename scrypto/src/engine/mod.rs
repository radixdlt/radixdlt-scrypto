/// Radix Engine System APIs.
pub mod api;
/// Types and functions shared by both Scrypto and Radix Engine.
pub mod types;

use sbor::Decode;

use crate::engine::api::RadixEngineInput;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: Decode>(input: RadixEngineInput) -> V {
    use crate::buffer::*;
    use crate::engine::api::radix_engine;

    unsafe {
        // 1. serialize the input
        let input_bytes = scrypto_encode_with_size_prefix(&input);

        // 2. make a radix engine call
        let output_ptr = radix_engine(input_bytes.as_ptr());

        // 3. deserialize the output
        scrypto_consume(output_ptr, |slice| scrypto_decode::<V>(slice).unwrap())
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: Decode>(_invocation: RadixEngineInput) -> V {
    todo!()
}
