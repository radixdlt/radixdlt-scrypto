use sbor::*;

use crate::buffer::*;
use crate::kernel::*;

pub fn call_kernel<T: Encode, V: Decode>(op: u32, input: T) -> V {
    // 1. serialize the input
    let input_bytes = radix_encode(&input);

    // 2. make a kernel call
    let output_ptr = unsafe { kernel_main(op, input_bytes.as_ptr(), input_bytes.len()) };

    // 3. copy and release the buffer (allocated by kernel)
    let output_bytes = radix_copy(output_ptr);
    radix_free(output_ptr);

    // 4. deserialize the output
    radix_decode(&output_bytes)
}
