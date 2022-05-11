/// Radix Engine System APIs.
pub mod api;
/// Types and functions shared by both Scrypto and Radix Engine.
pub mod types;

use sbor::*;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<T: Encode, V: Decode>(method: u32, arguments: T) -> V {
    use crate::buffer::*;
    use crate::engine::api::radix_engine;
    use crate::rust::vec;

    // TODO: introduce proper method name and encode arguments as array.
    let input = Value::Enum {
        name: method.to_string(),
        fields: vec![decode_any(&scrypto_encode(&arguments)).unwrap()],
    };

    unsafe {
        // 1. serialize the input
        let input_bytes = scrypto_encode_any_with_size_prefix(&input);

        // 2. make a radix engine call
        let output_ptr = radix_engine(input_bytes.as_ptr());

        // 3. deserialize the output
        scrypto_consume(output_ptr, |slice| scrypto_decode::<V>(slice).unwrap())
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<T: Encode, V: Decode>(_method: u32, _arguments: T) -> V {
    todo!()
}
