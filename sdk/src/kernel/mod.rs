extern crate alloc;
use alloc::vec::Vec;
use core::mem::forget;
use core::ptr::copy;

extern "C" {
    /// Entrance to radix kernel.
    pub fn radix_kernel(operation: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
}

//================
// Note that there is already an API for accessing global allocator, but it requires nightly build atm.
// See: https://doc.rust-lang.org/nightly/std/alloc/trait.Allocator.html
//================

/// Allocates a chunk of memory that is not tracked by Rust ownership system.
#[no_mangle]
pub extern "C" fn radix_alloc(length: usize) -> *mut u8 {
    unsafe {
        let mut buf = Vec::<u8>::with_capacity(4 + length);
        let ptr = buf.as_mut_ptr();
        forget(buf);

        copy((length as u32).to_le_bytes().as_ptr(), ptr, 4);
        ptr.offset(4)
    }
}

/// Measures the length of an allocated memory.
#[no_mangle]
pub extern "C" fn radix_measure(ptr: *mut u8) -> usize {
    unsafe {
        let mut length = [0u8; 4];
        copy(ptr.offset(-4), length.as_mut_ptr(), 4);
        u32::from_le_bytes(length) as usize
    }
}

/// Frees an allocated memory chunk.
#[no_mangle]
pub extern "C" fn radix_free(ptr: *mut u8) {
    unsafe {
        let length = radix_measure(ptr);
        let _drop_me = Vec::<u8>::from_raw_parts(ptr.offset(-4), 4 + length, 4 + length);
    }
}

/// Release an allocated memory chunk
pub fn radix_copy(ptr: *mut u8) -> Vec<u8> {
    unsafe {
        let length = radix_measure(ptr);
        let mut buf = Vec::with_capacity(length);
        copy(ptr, buf.as_mut_ptr(), length);
        buf.set_len(length);
        buf
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::*;

    #[test]
    fn test_memory_allocation() {
        let msg = "hello".as_bytes();
        let size = msg.len();

        // Test allocating memory
        let mut ptr = radix_alloc(size);
        assert_eq!(radix_measure(ptr), size);

        // Test copying memory
        unsafe {
            core::ptr::copy(msg.as_ptr(), ptr, size);
        }
        let copied = radix_copy(ptr);
        assert_eq!(copied, msg);

        // Ensure no memory leak
        for _ in 0..10 {
            radix_free(ptr);
            let ptr2 = radix_alloc(size);
            assert_eq!(ptr2, ptr);
            ptr = ptr2;
        }
    }
}
