extern "C" {
    fn add(x: i32, y: i32) -> i32;
    fn sub(x: i32, y: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn call_add(x: i32, y: i32) -> i32 {
    let product = unsafe { add(x, y) };
    product
}

#[no_mangle]
pub extern "C" fn call_sub(x: i32, y: i32) -> i32 {
    let product = unsafe { sub(x, y) };
    product
}
