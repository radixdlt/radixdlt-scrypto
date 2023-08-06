#[no_mangle]
pub fn add(x: u64, y: u64) -> u64 {
    x + y
}

#[no_mangle]
pub fn add_batch(x: u64, y: u64, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c += y;
    }
    c
}

#[no_mangle]
pub fn mul(x: u64, y: u64) -> u64 {
    x * y
}

#[no_mangle]
pub fn mul_batch(x: u64, y: u64, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c *= y;
    }
    c
}

#[no_mangle]
pub fn pow(x: u64, exp: u32) -> u64 {
    x.pow(exp)
}

#[no_mangle]
pub fn pow_batch(x: u64, exp: u32, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c = x.pow(exp);
    }
    c
}
