use sbor::*;

use crate::buffer::*;
use crate::kernel::*;
use crate::utils::*;

/// Utility function for making a kernel call.
pub fn call_kernel<T: Encode, V: Decode>(op: u32, input: T) -> V {
    // 1. serialize the input
    let input_bytes = scrypto_encode(&input);

    // 2. make a kernel call
    let output_ptr = unsafe { kernel(op, input_bytes.as_ptr(), input_bytes.len()) };

    // 3. deserialize the output
    scrypto_consume(output_ptr, |slice| {
        unwrap_or_panic(scrypto_decode::<V>(slice))
    })
}
