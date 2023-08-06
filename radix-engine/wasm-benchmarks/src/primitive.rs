pub fn add(x: u64, y: u64) -> u64 {
    x + y
}

pub fn add_batch(x: u64, y: u64, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c += y;
    }
    c
}

pub fn mul(x: u64, y: u64) -> u64 {
    x * y
}

pub fn mul_batch(x: u64, y: u64, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c *= y;
    }
    c
}

pub fn pow(x: u64, exp: u32) -> u64 {
    x.pow(exp)
}

pub fn pow_batch(x: u64, exp: u32, cnt: u32) -> u64 {
    let mut c = x;
    for _ in 0..cnt {
        c = x.pow(exp);
    }
    c
}
