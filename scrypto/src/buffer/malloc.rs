extern crate alloc;
use alloc::vec::Vec;
use core::mem::forget;
use core::ptr::copy;

//================
// Note that there is already an API for accessing global allocator, but it requires nightly build atm.
// See: https://doc.rust-lang.org/nightly/std/alloc/trait.Allocator.html
//================

const WORD: usize = core::mem::size_of::<usize>();

/// Allocates a chunk of memory that is not tracked by Rust ownership system.
#[no_mangle]
pub extern "C" fn radix_alloc(length: usize) -> *mut u8 {
    unsafe {
        let mut buf = Vec::<u8>::with_capacity(WORD + length);
        let ptr = buf.as_mut_ptr();
        forget(buf);

        copy(length.to_le_bytes().as_ptr(), ptr, WORD);
        ptr.offset(WORD as isize)
    }
}

/// Makes a copy of the memory chunk
pub fn radix_copy(ptr: *const u8) -> Vec<u8> {
    unsafe {
        let length = radix_measure(ptr);
        let mut buf = Vec::with_capacity(length);
        copy(ptr, buf.as_mut_ptr(), length);
        buf.set_len(length);
        buf
    }
}

/// Measures the length of an allocated memory.
#[no_mangle]
pub extern "C" fn radix_measure(ptr: *const u8) -> usize {
    unsafe {
        let mut length = [0u8; WORD];
        copy(ptr.offset(-(WORD as isize)), length.as_mut_ptr(), WORD);
        usize::from_le_bytes(length) as usize
    }
}

/// Frees an allocated memory chunk.
#[no_mangle]
pub extern "C" fn radix_free(ptr: *mut u8) {
    unsafe {
        let length = radix_measure(ptr);
        let _drop_me =
            Vec::<u8>::from_raw_parts(ptr.offset(-(WORD as isize)), WORD + length, WORD + length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for _ in 0..1000 {
            radix_free(ptr);
            ptr = radix_alloc(100_000_000);
        }
    }
}
