use radix_common::math::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn precise_decimal_add_native(a_ptr: *mut u8, b_ptr: *mut u8, c_ptr: *mut u8) -> u64;
    pub fn precise_decimal_mul_native(a_ptr: *mut u8, b_ptr: *mut u8, c_ptr: *mut u8) -> u64;
    pub fn precise_decimal_pow_native(a_ptr: *mut u8, b_ptr: *mut u8, c_ptr: *mut u8) -> u64;
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn precise_decimal_add_native(_a_ptr: *mut u8, _b_ptr: *mut u8, _c_ptr: *mut u8) -> u64 {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn precise_decimal_mul_native(_a_ptr: *mut u8, _b_ptr: *mut u8, _c_ptr: *mut u8) -> u64 {
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn precise_decimal_pow_native(_a_ptr: *mut u8, _b_ptr: *mut u8, _c_ptr: *mut u8) -> u64 {
    unreachable!()
}

#[no_mangle]
pub fn precise_decimal_add() -> i64 {
    let x = PreciseDecimal::ONE;
    let y = PreciseDecimal::ONE;
    let z = x.checked_add(y).unwrap();
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul() -> i64 {
    let x = PreciseDecimal::ONE;
    let y = PreciseDecimal::ONE;
    let z = x.checked_mul(y).unwrap();
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_pow() -> i64 {
    let x = PreciseDecimal::from(2);
    let exp = 20;
    let z = x.checked_powi(exp).unwrap();
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_add_call_native() -> i64 {
    let x = PreciseDecimal::ONE;
    let mut x_vec = x.to_vec();
    let x_ptr = x_vec.as_mut_ptr();

    let y = PreciseDecimal::ONE;
    let mut y_vec = y.to_vec();
    let y_ptr = y_vec.as_mut_ptr();

    let mut z_vec = Vec::<u8>::with_capacity(PreciseDecimal::BITS / 8);
    let z_ptr = z_vec.as_mut_ptr();

    unsafe {
        precise_decimal_add_native(x_ptr, y_ptr, z_ptr);
        z_vec.set_len(PreciseDecimal::BITS / 8);
    };

    let z = PreciseDecimal::try_from(&z_vec[..]).unwrap();
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul_call_native() -> i64 {
    let x = PreciseDecimal::ONE;
    let mut x_vec = x.to_vec();
    let x_ptr = x_vec.as_mut_ptr();

    let y = PreciseDecimal::ONE;
    let mut y_vec = y.to_vec();
    let y_ptr = y_vec.as_mut_ptr();

    let mut z_vec = Vec::<u8>::with_capacity(PreciseDecimal::BITS / 8);
    let z_ptr = z_vec.as_mut_ptr();

    unsafe {
        precise_decimal_mul_native(x_ptr, y_ptr, z_ptr);
        z_vec.set_len(PreciseDecimal::BITS / 8);
    };

    let z = PreciseDecimal::try_from(&z_vec[..]).unwrap();
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_pow_call_native() -> i64 {
    let x = PreciseDecimal::from(2);
    let mut x_vec = x.to_vec();
    let x_ptr = x_vec.as_mut_ptr();

    let y = 20u32;

    let mut z_vec = Vec::<u8>::with_capacity(PreciseDecimal::BITS / 8);
    let z_ptr = z_vec.as_mut_ptr();

    unsafe {
        precise_decimal_pow_native(x_ptr, y as *mut u8, z_ptr);
        z_vec.set_len(PreciseDecimal::BITS / 8);
    };

    let z = PreciseDecimal::try_from(&z_vec[..]).unwrap();
    z.is_positive().into()
}
