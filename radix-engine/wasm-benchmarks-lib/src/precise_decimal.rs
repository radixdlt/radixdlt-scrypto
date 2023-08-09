use radix_engine_common::math::PreciseDecimal;

#[no_mangle]
pub fn precise_decimal_add() -> i64 {
    let x = PreciseDecimal::ONE;
    let y = PreciseDecimal::ONE;
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul() -> i64 {
    let x = PreciseDecimal::ONE;
    let y = PreciseDecimal::ONE;
    let z = x * y;
    z.is_positive().into()
}
