pub mod api;
pub mod types;

use crate::buffer::*;
use api::*;
use sbor::*;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<T: Encode, V: Decode>(op: u32, input: T) -> V {
    unsafe {
        // 1. serialize the input
        let input_bytes = scrypto_encode(&input);

        // 2. make a radix engine call
        let output_ptr = radix_engine(op, input_bytes.as_ptr(), input_bytes.len());

        // 3. deserialize the output
        scrypto_consume(output_ptr, |slice| scrypto_decode::<V>(slice).unwrap())
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<T: Encode, V: Decode>(op: u32, input: T) -> V {
    if op == EMIT_LOG {
        let input_bytes = scrypto_encode(&input);
        #[allow(unused_variables)]
        let input_value = scrypto_decode::<EmitLogInput>(&input_bytes).unwrap();
        #[cfg(feature = "std")]
        println!("{}", input_value.message);
        let output_bytes = scrypto_encode(&EmitLogOutput {});
        scrypto_decode::<V>(&output_bytes).unwrap()
    } else {
        todo!()
    }
}
