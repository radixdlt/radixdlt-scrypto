use radix_engine_common::math::PreciseDecimal;

#[no_mangle]
pub fn precise_decimal_add(x: i64, y: i64, _cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
    let y = PreciseDecimal::from(y);
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_add_no_conversion(_x: i64, _y: i64, _cnt: i64) -> i64 {
    let x = -PreciseDecimal::ONE;
    let y = PreciseDecimal::MAX;
    let z = x + y;
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_add_batch(x: i64, y: i64, cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
    let y = PreciseDecimal::from(y);
    let mut z = x;
    for _ in 0..cnt {
        z += y;
    }
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul(x: i64, y: i64, _cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
    let y = PreciseDecimal::from(y);
    let z = x * y;
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul_no_conversion(_x: i64, _y: i64, _cnt: i64) -> i64 {
    let x = -PreciseDecimal::ONE;
    let y = PreciseDecimal::MAX;
    let z = x * y;
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_mul_batch(x: i64, y: i64, cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
    let y = PreciseDecimal::from(y);
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
pub fn precise_decimal_pow(x: i64, exp: i64, _cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
    let z = x.powi(exp);
    z.is_positive().into()
}

#[no_mangle]
pub fn precise_decimal_pow_batch(x: i64, exp: i64, cnt: i64) -> i64 {
    let x = PreciseDecimal::from(x);
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
fn prec_dec_fib(n: PreciseDecimal) -> PreciseDecimal {
    let n = PreciseDecimal::from(n);
    if n == PreciseDecimal::ONE || n == PreciseDecimal::ZERO {
        PreciseDecimal::ONE
    } else {
        prec_dec_fib(n - 1) + prec_dec_fib(n - 2)
    }
}

#[no_mangle]
pub fn precise_decimal_fib(n: i64, _: i64, _: i64) -> i64 {
    let n = PreciseDecimal::from(n);
    prec_dec_fib(n);
    0
}
