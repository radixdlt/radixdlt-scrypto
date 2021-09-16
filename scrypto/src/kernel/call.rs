use sbor::*;

use crate::buffer::*;
use crate::kernel::*;
use crate::utils::*;

/// Utility function for making a kernel call.
#[cfg(target_arch = "wasm32")]
pub fn call_kernel<T: Encode, V: Decode>(op: u32, input: T) -> V {
    unsafe {
        // 1. serialize the input
        let input_bytes = scrypto_encode(&input);

        // 2. make a kernel call
        let output_ptr = kernel(op, input_bytes.as_ptr(), input_bytes.len());

        // 3. deserialize the output
        scrypto_consume(output_ptr, |slice| unwrap_light(scrypto_decode::<V>(slice)))
    }
}

/// Utility function for making a kernel call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_kernel<T: Encode, V: Decode>(op: u32, input: T) -> V {
    if op == EMIT_LOG {
        let input_bytes = scrypto_encode(&input);
        #[allow(unused_variables)]
        let input_value = unwrap_light(scrypto_decode::<EmitLogInput>(&input_bytes));
        #[cfg(feature = "std")]
        println!("{}", input_value.message);
        let output_bytes = scrypto_encode(&EmitLogOutput {});
        unwrap_light(scrypto_decode::<V>(&output_bytes))
    } else {
        todo!()
    }
}
