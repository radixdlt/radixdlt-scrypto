#[no_mangle]
pub fn primitive_add() -> i64 {
    let x = 1i64;
    let y = 1i64;
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn primitive_mul() -> i64 {
    let x = 1i64;
    let y = 1i64;
    let z = x * y;
    z.is_positive().into()
}
