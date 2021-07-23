use serde::{de::DeserializeOwned, Serialize};

pub fn syscall<'de, T: Serialize, V: DeserializeOwned>(op: u32, input: T) -> V {
    // 1. serialize the input
    let input_bytes = crate::buffer::bincode_encode(&input);

    // 2. make a kernel call

    let output_ptr =
        unsafe { crate::kernel::radix_kernel(op, input_bytes.as_ptr(), input_bytes.len()) };
    // 3. copy and release the buffer (allocated by kernel)
    let output_bytes = crate::kernel::radix_copy(output_ptr);
    crate::kernel::radix_free(output_ptr);

    // 4. deserialize the output
    let output = crate::buffer::bincode_decode(&output_bytes);
    output
}
