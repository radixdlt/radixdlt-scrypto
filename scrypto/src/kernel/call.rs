use sbor::*;

use crate::buffer::*;
use crate::kernel::*;

/// Utility function for making a kernel call.
pub fn call_kernel<T: Encode, V: Decode>(op: u32, input: T) -> V {
    // 1. serialize the input
    let input_bytes = scrypto_encode(&input);

    // 2. make a kernel call
    let output_ptr = unsafe { kernel(op, input_bytes.as_ptr(), input_bytes.len()) };

    // 3. deserialize the output
    let output = scrypto_consume(output_ptr, |slice| scrypto_decode::<V>(slice).unwrap());

    output
}
