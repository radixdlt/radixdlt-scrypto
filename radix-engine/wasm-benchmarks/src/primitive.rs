#[no_mangle]
pub fn add(x: i64, y: i64) -> i64 {
    x + y
}

#[no_mangle]
pub fn add_batch(x: i64, y: i64, cnt: i32) -> i64 {
    let mut c = x;
    for _ in 0..cnt {
        c += y;
    }
    c
}

#[no_mangle]
pub fn mul(x: i64, y: i64) -> i64 {
    x * y
}

#[no_mangle]
pub fn mul_batch(x: i64, y: i64, cnt: i32) -> i64 {
    let mut c = x;
    for _ in 0..cnt {
        c *= y;
    }
    c
}

#[no_mangle]
pub fn pow(x: i64, exp: u32) -> i64 {
    x.pow(exp)
}

#[no_mangle]
pub fn pow_batch(x: i64, exp: u32, cnt: i32) -> i64 {
    let mut c = x;
    for _ in 0..cnt {
        c = x.pow(exp);
    }
    c
}
