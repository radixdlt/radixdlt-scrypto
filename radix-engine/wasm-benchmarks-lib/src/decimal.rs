use radix_engine_common::math::Decimal;

#[no_mangle]
pub fn decimal_add(x: i64, y: i64, _cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let y = Decimal::from(y);
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_add_no_conversion(_x: i64, _y: i64, _cnt: i64) -> i64 {
    let x = Decimal::ONE;
    let y = Decimal::ONE;
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_add_batch(x: i64, y: i64, cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let y = Decimal::from(y);
    let mut z = x;
    for _ in 0..cnt {
        z += y;
    }
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_mul(x: i64, y: i64, _cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let y = Decimal::from(y);
    let z = x * y;
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_mul_no_conversion(_x: i64, _y: i64, _cnt: i64) -> i64 {
    let x = Decimal::ONE;
    let y = Decimal::ONE;
    let z = x * y;
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_mul_batch(x: i64, y: i64, cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let y = Decimal::from(y);
    let mut z = x;
    for _ in 0..cnt {
        if z < x {
            z = z * y;
        } else {
            z = z * x;
        }
    }
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_pow(x: i64, exp: i64, _cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let z = x.powi(exp);
    z.is_positive().into()
}

#[no_mangle]
pub fn decimal_pow_batch(x: i64, exp: i64, cnt: i64) -> i64 {
    let x = Decimal::from(x);
    let mut c = x;
    for _ in 0..cnt {
        c = x.powi(exp);
        if c.is_positive() {
            c = -x;
        } else {
            c = x;
        }
    }
    c.is_positive().into()
}

#[no_mangle]
fn dec_fib(n: Decimal) -> Decimal {
    let n = Decimal::from(n);
    if n == Decimal::ONE || n == Decimal::ZERO {
        Decimal::ONE
    } else {
        dec_fib(n - 1) + dec_fib(n - 2)
    }
}

#[no_mangle]
pub fn decimal_fib(n: i64, _: i64, _: i64) -> i64 {
    let n = Decimal::from(n);
    dec_fib(n);
    0
}
