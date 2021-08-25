use crate::rust::mem::forget;
use crate::rust::ptr::copy;
use crate::rust::vec::Vec;

const WORD: usize = core::mem::size_of::<usize>();

/// Allocates a chunk of memory that is not tracked by Rust ownership system.
#[no_mangle]
pub extern "C" fn scrypto_alloc(length: usize) -> *mut u8 {
    unsafe {
        let mut buf = Vec::<u8>::with_capacity(WORD + length);
        let ptr = buf.as_mut_ptr();
        forget(buf);

        copy(length.to_le_bytes().as_ptr(), ptr, WORD);
        ptr.offset(WORD as isize)
    }
}

/// Measures the length of an allocated memory.
#[no_mangle]
pub extern "C" fn scrypto_measure(ptr: *const u8) -> usize {
    unsafe {
        let mut length = [0u8; WORD];
        copy(ptr.offset(-(WORD as isize)), length.as_mut_ptr(), WORD);
        usize::from_le_bytes(length) as usize
    }
}

/// Frees an allocated memory chunk.
#[no_mangle]
pub extern "C" fn scrypto_free(ptr: *mut u8) {
    scrypto_consume(ptr, |_| {});
}

/// Wraps a byte array into a memory chunk.
pub fn scrypto_wrap(value: &[u8]) -> *mut u8 {
    unsafe {
        let ptr = scrypto_alloc(value.len());
        copy(value.as_ptr(), ptr, value.len());
        ptr
    }
}

/// Consumes a memory chunk.
pub fn scrypto_consume<T>(ptr: *mut u8, f: fn(slice: &[u8]) -> T) -> T {
    unsafe {
        let length = scrypto_measure(ptr);
        let bytes =
            Vec::<u8>::from_raw_parts(ptr.offset(-(WORD as isize)), WORD + length, WORD + length);
        f(&bytes[WORD..])
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
        let ptr = scrypto_alloc(size);
        assert_eq!(scrypto_measure(ptr), size);
        scrypto_free(ptr);

        // Ensure no memory leak
        for _ in 0..1000 {
            let ptr = scrypto_alloc(100_000_000);
            scrypto_free(ptr);
        }
    }
}
