#[no_mangle]
pub fn primitive_add(x: i64, y: i64) -> i64 {
    x + y
}

#[no_mangle]
pub fn primitive_add_batch(x: i64, y: i64, cnt: i32) -> i64 {
    let mut c = x;
    for _ in 0..cnt {
        c += y;
    }
    c
}

#[no_mangle]
pub fn primitive_mul(x: i64, y: i64) -> i64 {
    x * y
}

#[no_mangle]
pub fn primitive_mul_batch(x: i64, y: i64, cnt: i32) -> i64 {
    let mut z = x;
    for _ in 0..cnt {
        if z < x {
            z = z * y;
        } else {
            z = z * x;
        }
    }
    z
}

#[no_mangle]
pub fn primitive_pow(x: i64, exp: u32) -> i64 {
    x.pow(exp)
}

#[no_mangle]
pub fn primitive_pow_batch(x: i64, exp: u32, cnt: i32) -> i64 {
    let mut c = x;
    for _ in 0..cnt {
        c = c.pow(exp);
        if c > 0 {
            c = -x
        } else {
            c = x;
        }
    }
    c
}

#[no_mangle]
fn prim_fib(n: i64) -> i64 {
    if n == 1 || n == 0 {
        1
    } else {
        prim_fib(n - 1) + prim_fib(n - 2)
    }
}

#[no_mangle]
pub fn primitive_fib(n: i64) {
    prim_fib(n);
}
