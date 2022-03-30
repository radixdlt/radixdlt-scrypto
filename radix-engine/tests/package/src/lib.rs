static mut MAX: [u8; 4] = (u32::MAX / 2).to_le_bytes();

#[no_mangle]
pub extern "C" fn Package_main() -> *mut u8 {
    unsafe {
        MAX.as_mut_ptr()
    }
}