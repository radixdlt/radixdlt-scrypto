use radix_engine_common::math::Decimal;

#[no_mangle]
pub fn decimal_add() -> i64 {
    let x = Decimal::ONE;
    let y = Decimal::ONE;
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_mul() -> i64 {
    let x = Decimal::ONE;
    let y = Decimal::ONE;
    let z = x * y;
    z.is_positive().into()
}
