use sbor::rust::mem::forget;
use sbor::rust::ptr::copy;
use sbor::rust::vec::Vec;

/// Allocates a chunk of memory that is not tracked by Rust ownership system.
///
/// # Safety
/// The allocated memory is not tracked by Rust ownership; be use to free it after use.
#[no_mangle]
pub unsafe extern "C" fn scrypto_alloc(len: u32) -> *mut u8 {
    let cap = (len + 4) as usize;
    let mut buf = Vec::<u8>::with_capacity(cap);
    let ptr = buf.as_mut_ptr();
    forget(buf);
    copy(len.to_le_bytes().as_ptr(), ptr, 4);
    ptr
}

/// Allocates a buffer with initial values.
pub fn scrypto_alloc_initialized(bytes: Vec<u8>) -> *mut u8 {
    unsafe {
        let ptr = scrypto_alloc(bytes.len() as u32);
        copy(bytes.as_ptr(), ptr.add(4), bytes.len());
        ptr
    }
}

/// Consumes a buffer.
///
/// # Safety
/// The memory pointer must be obtained from `scrypto_alloc` or `scrypto_wrap`.
pub fn scrypto_consume<T>(ptr: *mut u8, f: fn(slice: &[u8]) -> T) -> T {
    unsafe {
        let mut len = [0u8; 4];
        copy(ptr, len.as_mut_ptr(), 4);

        let cap = (u32::from_le_bytes(len) + 4) as usize;
        let buf = Vec::<u8>::from_raw_parts(ptr, cap, cap);
        f(&buf[4..])
    }
}

/// Releases a buffer.
///
/// # Safety
/// The memory pointer must be obtained from `scrypto_alloc` or `scrypto_wrap`.
#[no_mangle]
pub unsafe extern "C" fn scrypto_free(ptr: *mut u8) {
    scrypto_consume(ptr, |_| {});
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation() {
        let msg = "hello".as_bytes();
        let size = msg.len();

        unsafe {
            // Test allocating memory
            let ptr = scrypto_alloc(size as u32);
            scrypto_free(ptr);

            // Ensure no memory leak
            for _ in 0..1000 {
                let ptr = scrypto_alloc(100_000_000);
                scrypto_free(ptr);
            }
        }
    }
}
