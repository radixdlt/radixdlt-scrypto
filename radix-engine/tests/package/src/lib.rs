static mut LARGE: [u8; 4] = (u32::MAX / 2).to_le_bytes();
static mut MAX: [u8; 4] = u32::MAX.to_le_bytes();
static mut ZERO: [u8; 4] = [0, 0, 0, 0];

#[no_mangle]
pub extern "C" fn LargeReturnSize_main() -> *mut u8 {
    unsafe { LARGE.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_main() -> *mut u8 {
    unsafe { MAX.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_main() -> *mut u8 {
    unsafe { ZERO.as_mut_ptr() }
}
